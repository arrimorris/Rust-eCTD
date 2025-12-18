// trailblazer_app/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use sqlx::postgres::PgPoolOptions;
use trailblazer_compliance::submission::EctdService; // We assume this exists now
use aws_sdk_s3::Client as S3Client;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::Region;

// Minimal State placeholder
struct AppState {}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:ectd_password@localhost:5432/ectd_v4".to_string());

    // Minimal setup
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to Database");

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
