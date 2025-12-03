use std::env;
use dotenvy::dotenv;
use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_region: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv().ok(); // Load .env if present

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,

            s3_endpoint: env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),

            s3_bucket: env::var("S3_BUCKET")
                .unwrap_or_else(|_| "ectd-documents".to_string()),

            s3_region: env::var("AWS_REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
        })
    }
}
