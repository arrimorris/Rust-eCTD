use clap::Args;
use uuid::Uuid;
use std::path::PathBuf;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, config::Region};
use sqlx::PgPool;

use ectd_service::{EctdService, documents::AddDocumentParams};
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
    println!("📎 Attaching Document via Service Layer...");

    // 1. Init S3 (This ensures the CLI uses the same S3 logic as the Service expects)
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new(config.s3_region));
    let aws_config = aws_config::from_env().region(region_provider).load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(true)
        .endpoint_url(&config.s3_endpoint)
        .build();
    let s3_client = Client::from_conf(s3_config);

    // 2. Init Service
    let service = EctdService::new(pool, s3_client, config.s3_bucket);

    // 3. Delegate to Service
    let params = AddDocumentParams {
        submission_id: args.id,
        file_path: args.file,
        context_code: args.context,
        title: args.title,
        priority: args.priority,
    };

    let doc_id = service.attach_document(params).await?;

    println!("✅ Document Attached. UUID: {}", doc_id);
    Ok(())
}
