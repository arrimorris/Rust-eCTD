// ectd_cli/src/main.rs
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod commands;

#[derive(Parser)]
#[command(name = "ectd_forge")]
#[command(about = "Open Source eCTD v4.0 Toolchain", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest a submissionunit.xml file into the database
    Ingest(commands::ingest::IngestArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load environment variables (.env)
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    // 2. Connect to the Brain (Postgres)
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // 3. Parse arguments and route to the correct command
    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest(args) => {
            commands::ingest::execute(pool, args).await?;
        }
    }

    Ok(())
}
