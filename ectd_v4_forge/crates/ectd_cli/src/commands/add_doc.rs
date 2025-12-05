use clap::Args;
use sqlx::PgPool;
use uuid::Uuid;
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::Read;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, config::Region};
use aws_sdk_s3::primitives::ByteStream;

use ectd_core::models::{
    document::{Document, DocumentTitle, DocumentText, DocumentReferencePath},
    context_of_use::{ContextOfUse, PriorityNumber, DocumentReference, DocumentIdRef},
};
use ectd_db::repository::SubmissionRepository;
use crate::config::Config;

#[derive(Debug, Args)]
pub struct AddDocArgs {
    /// The Submission Unit UUID to attach this file to
    #[arg(short, long)]
    pub id: Uuid,

    /// Path to the physical file (e.g. ./cover.pdf)
    #[arg(short, long)]
    pub file: PathBuf,

    /// The eCTD Context Code (e.g. "cover-letter", "clinical-study-report")
    #[arg(short, long)]
    pub context: String,

    /// The Document Title (e.g. "Cover Letter", "Study 101 Report")
    #[arg(short, long)]
    pub title: String,

    /// Priority Number (Default: 1)
    #[arg(long, default_value_t = 1)]
    pub priority: u32,
}

pub async fn execute(pool: PgPool, config: Config, args: AddDocArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("📎 Attaching Document to Submission: {}", args.id);
    println!("   File: {:?}", args.file);

    // 1. Validation (Fail Fast)
    if !args.file.exists() {
        return Err(format!("File not found: {:?}", args.file).into());
    }

    // 2. Calculate Checksum (Rule eCTD4-048)
    println!("   Calculating SHA-256 Checksum...");
    let mut file_obj = File::open(&args.file)?;
    let mut hasher = Sha256::new();
    // Manual read loop because Sha256 doesn't implement Write
    let mut buffer = [0; 1024 * 32]; // 32KB buffer
    loop {
        let count = file_obj.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let hash = hex::encode(hasher.finalize());
    println!("   SHA-256: {}", hash);

    // 3. Generate Identity (UUIDv4)
    let doc_id = Uuid::new_v4();
    let cou_id = Uuid::new_v4();

    // 4. Upload to S3 (The Vault)
    println!("📤 Uploading to S3...");
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new(config.s3_region.clone()));
    let aws_config = aws_config::from_env().region(region_provider).load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(true)
        .endpoint_url(&config.s3_endpoint)
        .build();
    let s3_client = Client::from_conf(s3_config);

    // Stream upload to save memory
    let body = ByteStream::from_path(&args.file).await?;

    s3_client.put_object()
        .bucket(&config.s3_bucket)
        .key(doc_id.to_string())
        .body(body)
        .content_type("application/pdf") // Simplification: assume PDF for now
        .send()
        .await?;

    // 5. Construct Models (The Core)
    let filename = args.file.file_name().unwrap().to_string_lossy().to_string();
    // Default folder structure: m1/us/filename
    // In a future phase, we can map context codes to specific folders (m2, m3, etc.)
    let ref_path = format!("m1/us/{}", filename);

    let doc = Document {
        id: doc_id.to_string(),
        title: DocumentTitle { value: args.title },
        text: DocumentText {
            reference: DocumentReferencePath { value: ref_path },
            checksum: hash,
            checksum_algorithm: "SHA256".to_string(),
            media_type: "application/pdf".to_string(),
        },
    };

    let cou = ContextOfUse {
        id: cou_id.to_string(),
        code: args.context,
        code_system: "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string(),
        status_code: "active".to_string(),
        priority_number: PriorityNumber { value: args.priority },
        document_reference: Some(DocumentReference {
            id: DocumentIdRef { root: doc_id.to_string() }
        }),
        related_context_of_use: None,
        keywords: vec![],
    };

    // 6. Update Database (The Brain)
    let repo = SubmissionRepository::new(pool);
    repo.add_document_to_submission(args.id, &doc, &cou).await?;

    println!("✅ Document Attached Successfully.");
    Ok(())
}
