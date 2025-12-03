use clap::Args;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Args, Debug)]
pub struct ImportStandardArgs {
    /// Path to the CDASHIG CSV file (e.g., CDASHIG_v2.3.csv)
    #[arg(short, long)]
    pub file: String,

    /// Output directory for the JSON maps
    #[arg(short, long, default_value = "./mappings")]
    pub output: String,
}

// The output JSON structure
#[derive(Debug, Serialize, Clone)]
struct DomainMap {
    domain: String,
    description: String,
    variables: Vec<VariableMap>,
}

#[derive(Debug, Serialize, Clone)]
struct VariableMap {
    cdash: String,
    sdtm: String,
    role: String,
    notes: String,
}

pub fn run(args: ImportStandardArgs) -> anyhow::Result<()> {
    println!("⚙️  Parsing Standard: {}", args.file);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(&args.file)?;

    // We group everything by Domain (e.g., "AE", "DM")
    let mut domains: HashMap<String, DomainMap> = HashMap::new();

    for result in rdr.records() {
        let record = result?;
        
        // Based on your CSV structure:
        // Col 2: Domain (e.g., "AG")
        // Col 5: CDASHIG Variable (e.g., "STUDYID")
        // Col 13: SDTMIG Target (e.g., "STUDYID")
        // Col 6: Label/Notes
        
        let domain_code = record.get(2).unwrap_or("Unknown").to_string();
        let cdash_var = record.get(5).unwrap_or("").to_string();
        let sdtm_target = record.get(13).unwrap_or("").to_string();
        let label = record.get(6).unwrap_or("").to_string();

        // Skip if there is no SDTM target (internal CDASH-only fields)
        if sdtm_target.is_empty() {
            continue;
        }

        let entry = domains.entry(domain_code.clone()).or_insert(DomainMap {
            domain: domain_code.clone(),
            description: format!("Imported {} Domain", domain_code),
            variables: Vec::new(),
        });

        entry.variables.push(VariableMap {
            cdash: cdash_var,
            sdtm: sdtm_target,
            role: "Imported".to_string(), // We could parse Col 11 ("Core") for this
            notes: label,
        });
    }

    // Write them to disk
    fs::create_dir_all(&args.output)?;
    
    for (code, map) in domains {
        if code.is_empty() { continue; }
        
        let file_path = Path::new(&args.output).join(format!("{}_map.json", code.to_lowercase()));
        let json = serde_json::to_string_pretty(&map)?;
        fs::write(&file_path, json)?;
        println!("   ✅ Generated map: {}", file_path.display());
    }

    println!("✨ Import Complete. You can now use 'ingest' with these domains.");
    Ok(())
}
