use clap::Args;
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use sqlx::PgPool;
use uuid::Uuid;
use ectd_db::repository::SubmissionRepository;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, config::Region};
use sha2::{Sha256, Digest};
use crate::config::Config;
use futures::stream::{self, StreamExt}; // For parallelism
use tokio::io::AsyncWriteExt; // For async file writing

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// The UUID of the submission unit to export
    #[arg(short, long)]
    pub id: Uuid,

    /// The output directory (e.g. ./output/0001)
    #[arg(short, long)]
    pub output: PathBuf,
}

pub async fn execute(pool: PgPool, config: Config, args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Starting Export for Submission: {}", args.id);
    println!("📂 Output Directory: {:?}", args.output);

    // 1. Fetch Data (The Brain)
    let repo = SubmissionRepository::new(pool);
    let unit = repo.get_submission(args.id).await
        .map_err(|e| format!("Failed to fetch submission from DB: {}", e))?;

    println!("✅ Metadata retrieved. Found {} documents.", unit.documents.len());

    // 2. Prepare Output Directory
    if args.output.exists() {
        println!("⚠️  Warning: Output directory exists. Overwriting content.");
    }
    fs::create_dir_all(&args.output)?;

    // 3. Setup S3 Client (The Vault)
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new(config.s3_region.clone()));
    let aws_config = aws_config::from_env().region(region_provider).load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(true)
        .endpoint_url(&config.s3_endpoint)
        .build();
    let s3_client = Client::from_conf(s3_config);
    let bucket_name = config.s3_bucket.clone();

    // Track files for the manifest (sha256.txt) - Protected by Mutex/Channel or handled via collection
    // Since we stream and collect results, we can build the manifest from the results.

    // 4. Download Physical Files (Parallelized)
    println!("⬇️  Downloading {} files concurrently...", unit.documents.len());

    // Create a stream of async tasks
    let downloads = stream::iter(unit.documents.iter())
        .map(|doc| {
            let client = s3_client.clone();
            let bucket = bucket_name.clone();
            let out_dir = args.output.clone();
            let doc_id = doc.id.clone();
            let ref_path = doc.text.reference.value.clone();

            async move {
                let rel_path = Path::new(&ref_path);
                let full_out_path = out_dir.join(rel_path);

                if let Some(parent) = full_out_path.parent() {
                    fs::create_dir_all(parent)?; // Sync fs is okay for directory creation
                }

                // S3 Get
                let mut stream = client
                    .get_object()
                    .bucket(bucket)
                    .key(&doc_id)
                    .send()
                    .await?
                    .body;

                let mut file = tokio::fs::File::create(&full_out_path).await?;

                while let Some(bytes) = stream.try_next().await? {
                    file.write_all(&bytes).await?;
                }

                // Return path and relative string for hashing
                Ok::<(PathBuf, String), Box<dyn std::error::Error + Send + Sync>>((full_out_path, ref_path))
            }
        })
        .buffer_unordered(10); // Process 10 concurrent downloads

    // Collect results
    let results = downloads.collect::<Vec<_>>().await;

    // Process results for manifest
    let mut manifest_entries: Vec<(String, String)> = Vec::new();

    for res in results {
        match res {
            Ok((path, rel_str)) => {
                let hash = calculate_file_hash(&path)?; // CPU-bound hashing done synchronously
                manifest_entries.push((hash, rel_str.replace("\\", "/")));
            }
            Err(e) => return Err(format!("Download failed: {}", e).into()),
        }
    }

    // 5. Generate submissionunit.xml
    println!("📝 Generating submissionunit.xml...");
    let final_xml = unit.to_xml()
        .map_err(|e| format!("XML Serialization Failed: {}", e))?;

    let xml_path = args.output.join("submissionunit.xml");
    fs::write(&xml_path, &final_xml)?;

    // Add XML to manifest
    let xml_hash = calculate_file_hash(&xml_path)?;
    manifest_entries.push((xml_hash, "submissionunit.xml".to_string()));

    // 6. Generate sha256.txt Manifest
    println!("Calculated Hashes for Manifest:");
    let manifest_path = args.output.join("sha256.txt");
    let mut manifest_file = BufWriter::new(File::create(manifest_path)?);

    for (hash, filename) in manifest_entries {
        writeln!(manifest_file, "{}  {}", hash, filename)?;
    }

    println!("🎉 Export Complete! Package ready at: {:?}", args.output);
    Ok(())
}

// Helper: Synchronous Hashing (CPU bound)
fn calculate_file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}
