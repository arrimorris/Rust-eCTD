use clap::Args;
use std::path::PathBuf;
use sqlx::PgPool;
use uuid::Uuid;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, config::Region};
use crate::config::Config;
use ectd_service::EctdService;
use futures::StreamExt; // For iterating the stream
// Removed pin_mut import as we use BoxStream now

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

    // 1. Setup Service (The Engine)
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new(config.s3_region.clone()));
    let aws_config = aws_config::from_env().region(region_provider).load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(true)
        .endpoint_url(&config.s3_endpoint)
        .build();
    let s3_client = Client::from_conf(s3_config);
    let bucket_name = config.s3_bucket.clone();

    let service = EctdService::new(pool, s3_client, bucket_name);

    // 2. Consume Stream
    let mut stream = service.export_submission_stream(args.id, args.output.clone());

    while let Some(result) = stream.next().await {
        match result {
            Ok(progress) => {
                // Simple Progress Indicator
                // In a real CLI, we might use indicatif here
                if progress.total_files > 0 {
                    let percent = (progress.processed_files as f64 / progress.total_files as f64) * 100.0;
                    println!(
                        "[{:>3.0}%] {} - {}",
                        percent,
                        progress.status,
                        progress.file_name
                    );
                } else {
                     println!("{} - {}", progress.status, progress.file_name);
                }
            }
            Err(e) => {
                return Err(format!("Export Failed: {}", e).into());
            }
        }
    }

    println!("🎉 Export Complete! Package ready at: {:?}", args.output);
    Ok(())
}
