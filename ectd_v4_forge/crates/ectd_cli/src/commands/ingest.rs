use clap::Args;
use std::fs;
use std::path::PathBuf;
use sqlx::PgPool;
use quick_xml::de::from_str;
use ectd_core::models::submission_unit::SubmissionUnit;
use ectd_db::repository::SubmissionRepository;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, config::Region};
use aws_sdk_s3::primitives::ByteStream; // For streaming uploads

#[derive(Debug, Args)]
pub struct IngestArgs {
    /// The path to the submissionunit.xml file you want to import
    #[arg(short, long)]
    pub file: PathBuf,

    /// (Optional) The submission ID to override/link if not in the XML
    #[arg(short, long)]
    pub submission_id: Option<String>,
}

pub async fn execute(pool: PgPool, args: IngestArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting ingestion for: {:?}", args.file);

    // 0. SETUP S3 CLIENT (MinIO)
    // We load credentials from the environment (standard AWS_ACCESS_KEY_ID style)
    // or fallback to our docker defaults if running locally.
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
    let config = aws_config::from_env().region(region_provider).load().await;

    // We must force "Path Style" addressing for MinIO (localhost compatibility)
    let s3_config_builder = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .endpoint_url(std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string()));

    let s3_client = Client::from_conf(s3_config_builder.build());
    let bucket_name = "ectd-documents";

    // 1. READ THE XML FILE
    let xml_content = fs::read_to_string(&args.file)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Calculate the "Base Directory" so we can find the PDFs relative to the XML
    let base_dir = args.file.parent().unwrap_or_else(|| std::path::Path::new("."));

    // 2. PARSE THE XML
    let mut submission_unit: SubmissionUnit = from_str(&xml_content)
        .map_err(|e| format!("XML Parsing Error: {}", e))?;

    if let Some(sub_id) = args.submission_id {
        println!("🔧 Overriding Submission ID with: {}", sub_id);
        submission_unit.submission.id = sub_id;
    }

    println!("✅ XML Parsed. Preparing to upload {} documents...", submission_unit.documents.len());

    // 3. UPLOAD FILES TO S3 (The "Vault")
    for doc in &submission_unit.documents {
        let rel_path = &doc.text.reference.value;
        let file_path = base_dir.join(rel_path);
        let s3_key = &doc.id; // Store by UUID (Content Addressable)

        println!("📤 Uploading: {} -> s3://{}/{}", rel_path, bucket_name, s3_key);

        // Verify file exists locally before trying
        if !file_path.exists() {
            eprintln!("❌ FILE MISSING: {:?}", file_path);
            return Err(format!("Required file not found: {:?}", file_path).into());
        }

        // Stream the file to S3 (Low memory usage)
        let body = ByteStream::from_path(&file_path).await;
        match body {
            Ok(b) => {
                s3_client
                    .put_object()
                    .bucket(bucket_name)
                    .key(s3_key)
                    .body(b)
                    .content_type("application/pdf") // Force PDF mime type
                    .send()
                    .await?;
            },
            Err(e) => return Err(format!("Failed to open file for streaming: {}", e).into())
        }
    }
    println!("📦 All files uploaded to Storage.");

    // 4. COMMIT TO POSTGRES (The "Brain")
    let repo = SubmissionRepository::new(pool);
    println!("💾 Committing metadata to Database...");

    match repo.create_submission(&submission_unit).await {
        Ok(id) => {
            println!("🎉 SUCCESS! Submission Unit fully ingested.");
            println!("🔑 Primary Key (UUID): {}", id);
        },
        Err(e) => {
            // NOTE: In a real prod system, you might want to delete the S3 files here to cleanup
            eprintln!("❌ DATABASE ERROR: Transaction rolled back.");
            eprintln!("Reason: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
