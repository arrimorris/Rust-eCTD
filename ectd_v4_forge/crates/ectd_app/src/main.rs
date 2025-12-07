// crates/ectd_app/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state; // Import the new state module

use sqlx::postgres::PgPoolOptions;
use ectd_service::EctdService;
use aws_sdk_s3::Client as S3Client;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::Region;
use crate::state::AppState; // Use the AppState

#[tokio::main]
async fn main() {
    // 1. Setup Configuration (Defaults to Docker localhost)
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:ectd_password@localhost:5432/ectd_v4".to_string());

    let s3_endpoint = std::env::var("S3_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());

    let s3_bucket = "ectd-documents".to_string();

    // 2. Connect to the "Brain" (Postgres)
    println!("🔌 Connecting to Database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to Database (Is Docker running?)");

    // 3. ENSURE SCHEMA INTEGRITY (Build Order Architecture)
    // The app carries the schema blueprint and enforces it on startup.
    println!("🔄 Ensuring Schema Integrity...");
    ectd_db::schema::rebuild_database(&pool)
        .await
        .expect("Failed to apply schema");

    // 4. Connect to the "Vault" (MinIO)
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
    let aws_config = aws_config::from_env().region(region_provider).load().await;
    let s3_config = aws_sdk_s3::config::Builder::from(&aws_config)
        .force_path_style(true)
        .endpoint_url(&s3_endpoint)
        .build();
    let s3_client = S3Client::from_conf(s3_config);

    // 5. Initialize Service & State
    let service = EctdService::new(pool, s3_client, s3_bucket);
    let app_state = AppState::new();

    // 6. Launch Tauri
    tauri::Builder::default()
        .manage(service) // Inject the Business Logic
        .manage(app_state) // Inject the System Logic
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::init_submission,
            commands::add_document,
            commands::validate_submission,
            commands::export_submission,
            commands::ensure_infrastructure, // Register the new command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
