use clap::Args;
use sqlx::PgPool;
use sqlx::migrate::Migrator;

// Embed migrations from the ectd_db crate
// Note: Path is relative to CARGO_MANIFEST_DIR of ectd_cli
static MIGRATOR: Migrator = sqlx::migrate!("../ectd_db/migrations");

#[derive(Debug, Args)]
pub struct MigrateArgs {
    /// Revert the latest migration (rollback)
    #[arg(long)]
    pub revert: bool,
}

pub async fn execute(pool: PgPool, args: MigrateArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Checking Database Schema Status...");

    if args.revert {
        println!("⏪ Reverting last migration...");
        MIGRATOR.undo(&pool, 1).await?; // Undo 1 step
        println!("✅ Revert complete.");
    } else {
        println!("🚀 Applying pending migrations...");
        MIGRATOR.run(&pool).await?;
        println!("✅ Database is up to date.");
    }

    Ok(())
}
