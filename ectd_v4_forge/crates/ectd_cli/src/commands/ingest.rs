use clap::Args;
use std::fs;
use std::path::PathBuf;
use sqlx::PgPool;
use quick_xml::de::from_str;
use ectd_core::models::submission_unit::SubmissionUnit;
use ectd_db::repository::SubmissionRepository;

#[derive(Debug, Args)]
pub struct IngestArgs {
    /// The path to the submissionunit.xml file you want to import
    #[arg(short, long)]
    pub file: PathBuf,

    /// (Optional) The submission ID to override/link if not in the XML
    #[arg(short, long)]
    pub submission_id: Option<String>,
}

pub async fn execute(pool: PgPool, args: IngestArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting ingestion for: {:?}", args.file);

    // 1. READ THE FILE
    // We load the raw XML string from disk.
    let xml_content = fs::read_to_string(&args.file)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    println!("📄 File read successfully. Size: {} bytes", xml_content.len());

    // 2. PARSE THE XML (The Kinetic Energy)
    // This converts the text into our Rust 'SubmissionUnit' struct.
    // If the XML is malformed or missing fields required by the struct, this fails fast.
    let mut submission_unit: SubmissionUnit = from_str(&xml_content)
        .map_err(|e| format!("XML Parsing Error: {}", e))?;

    // Quick fix: If the user provided a submission_id flag, override it here.
    if let Some(sub_id) = args.submission_id {
        println!("🔧 Overriding Submission ID with: {}", sub_id);
        submission_unit.submission.id = sub_id;
    }

    println!("✅ XML Parsed. Found {} documents and {} context of use elements.",
        submission_unit.documents.len(),
        submission_unit.context_of_use.len()
    );

    // 3. COMMIT TO POSTGRES (The Potential Energy)
    // We initialize the repository and run the transactional insert.
    let repo = SubmissionRepository::new(pool);

    println!("💾 Beginning database transaction...");
    match repo.create_submission(&submission_unit).await {
        Ok(id) => {
            println!("🎉 SUCCESS! Submission Unit committed.");
            println!("🔑 Primary Key (UUID): {}", id);
        },
        Err(e) => {
            eprintln!("❌ DATABASE ERROR: Transaction rolled back.");
            eprintln!("Reason: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
