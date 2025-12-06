use clap::Args;
use sqlx::PgPool;
use ectd_db::schema::rebuild_database;

#[derive(Debug, Args)]
pub struct RebuildArgs {
    /// DANGER: Drop existing tables before rebuilding?
    #[arg(long)]
    pub reset: bool,
}

pub async fn execute(pool: PgPool, args: RebuildArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Rebuilding Database Schema...");

    if args.reset {
        println!("🔥 Reset requested. Dropping public schema...");
        sqlx::query!("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
            .execute(&pool).await?;
    }

    rebuild_database(&pool).await?;

    println!("✅ Database Schema Applied Successfully.");
    Ok(())
}
