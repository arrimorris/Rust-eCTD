use rust_embed::RustEmbed;
use sqlx::{PgPool, Executor};
use std::str;

#[derive(RustEmbed)]
#[folder = "schema/"]
struct SchemaAssets;

/// Reads the build order and applies all SQL files in a single transaction.
pub async fn rebuild_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Read the Manifest
    let manifest = get_file_content("00_build_order.sql")
        .expect("Missing 00_build_order.sql");

    // 2. Parse and Aggregate SQL
    let mut full_script = String::new();

    for line in manifest.lines() {
        let trimmed = line.trim();

        // Parse: -- @include folder/file.sql
        if let Some(path) = parse_include_directive(trimmed) {
            println!("   📄 Including: {}", path);
            let content = get_file_content(path)
                .expect(&format!("Missing included file: {}", path));
            full_script.push_str(&content);
            full_script.push('\n');
        } else if !trimmed.starts_with("--") {
            // Keep normal lines (if any), ignore comments
            full_script.push_str(line);
            full_script.push('\n');
        }
    }

    // 3. Execute
    tx.execute(full_script.as_str()).await?;
    tx.commit().await?;

    Ok(())
}

fn get_file_content(path: &str) -> Option<String> {
    SchemaAssets::get(path)
        .map(|f| str::from_utf8(f.data.as_ref()).unwrap().to_string())
}

fn parse_include_directive(line: &str) -> Option<&str> {
    if line.starts_with("--") && line.contains("@include") {
        line.split_whitespace().last()
    } else {
        None
    }
}
