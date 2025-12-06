// ectd_cli/src/main.rs
use clap::{Parser, Subcommand};
use sqlx::postgres::PgPoolOptions;

use ectd_cli::commands;
use ectd_cli::config::Config;

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

    /// Validate a submissionunit.xml against FDA/ICH rules
    Validate(commands::validate::ValidateArgs),

    /// Import CDISC standards from CSV
    ImportStandard(commands::import_standard::ImportStandardArgs),

    /// Forge a SAS XPT v5 dataset from CSV
    ForgeData(commands::forge_data::ForgeDataArgs),

    /// Export a submission package from the database to disk
    Export(commands::export::ExportArgs),

    /// Rebuild the database schema from embedded assets
    Rebuild(commands::rebuild::RebuildArgs),

    /// Initialize a new empty submission with metadata
    Init(commands::init::InitArgs),

    /// Add a document to an existing submission
    AddDoc(commands::add_doc::AddDocArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load Config (Fails fast if invalid)
    let config = Config::from_env()?;

    // 3. Parse arguments and route to the correct command
    let cli = Cli::parse();

    match cli.command {
        Commands::Ingest(args) => {
            // 2. Connect to the Brain (Postgres)
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?;
            commands::ingest::execute(pool, config, args).await?;
        }
        Commands::Validate(args) => {
            // Note: Validate doesn't need the 'pool', keeping it pure logic.
            commands::validate::execute(args).await?;
        }
        Commands::ImportStandard(args) => {
            commands::import_standard::run(args)?;
        }
        Commands::ForgeData(args) => {
            commands::forge_data::run(args)?;
        }
        Commands::Export(args) => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?;
            commands::export::execute(pool, config, args).await?;
        }
        Commands::Rebuild(args) => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?;
            commands::rebuild::execute(pool, args).await?;
        }
        Commands::Init(args) => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?;
            commands::init::execute(pool, args).await?;
        }
        Commands::AddDoc(args) => {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&config.database_url)
                .await?;
            commands::add_doc::execute(pool, config, args).await?;
        }
    }

    Ok(())
}
