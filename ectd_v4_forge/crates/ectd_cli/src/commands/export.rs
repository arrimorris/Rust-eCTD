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
use quick_xml::se::to_string;

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// The UUID of the submission unit to export
    #[arg(short, long)]
    pub id: Uuid,

    /// The output directory (e.g. ./output/0001)
    #[arg(short, long)]
    pub output: PathBuf,
}

pub async fn execute(pool: PgPool, args: ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
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
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
    let config = aws_config::from_env().region(region_provider).load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .endpoint_url(std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()))
        .build();
    let s3_client = Client::from_conf(s3_config);
    let bucket_name = "ectd-documents";

    // Track files for the manifest (sha256.txt)
    let mut manifest_entries: Vec<(String, String)> = Vec::new();

    // 4. Download Physical Files
    for doc in &unit.documents {
        let rel_path = Path::new(&doc.text.reference.value);
        let full_out_path = args.output.join(rel_path);

        // Ensure parent folder exists (e.g. m1/us/)
        if let Some(parent) = full_out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        println!("⬇️  Downloading: {} (ID: {})", doc.text.reference.value, doc.id);

        let mut file = File::create(&full_out_path)?;

        // Stream from S3
        let mut stream = s3_client
            .get_object()
            .bucket(bucket_name)
            .key(&doc.id) // Key is the UUID
            .send()
            .await?
            .body;

        // Write stream to disk
        while let Some(bytes) = stream.try_next().await? {
            file.write_all(&bytes)?;
        }

        // Calculate Hash for Manifest (Rule eCTD4-062)
        let hash = calculate_file_hash(&full_out_path)?;
        manifest_entries.push((hash, doc.text.reference.value.clone()));
    }

    // 5. Generate submissionunit.xml
    println!("📝 Generating submissionunit.xml...");
    let xml_string = to_string(&unit)
        .map_err(|e| format!("XML Serialization Failed: {}", e))?;

    // Prepend XML declaration (quick-xml doesn't add it by default)
    let final_xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, xml_string);

    let xml_path = args.output.join("submissionunit.xml");
    fs::write(&xml_path, &final_xml)?;

    // Add XML to manifest
    let xml_hash = calculate_file_hash(&xml_path)?;
    manifest_entries.push((xml_hash, "submissionunit.xml".to_string()));

    // 6. Generate sha256.txt Manifest
    // Format: "hash  filename" (standard sha256sum format)
    println!("Calculated Hashes for Manifest:");
    let manifest_path = args.output.join("sha256.txt");
    let mut manifest_file = BufWriter::new(File::create(manifest_path)?);

    for (hash, filename) in manifest_entries {
        // eCTD usually expects forward slashes even on Windows
        let standardized_name = filename.replace("\\", "/");
        writeln!(manifest_file, "{}  {}", hash, standardized_name)?;
    }

    println!("🎉 Export Complete! Package ready at: {:?}", args.output);
    Ok(())
}

fn calculate_file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}
