use clap::Args;
use std::fs;
use std::path::PathBuf;
use quick_xml::de::from_str;
use ectd_core::models::submission_unit::SubmissionUnit;
use ectd_core::get_standard_validator;

#[derive(Debug, Args)]
pub struct ValidateArgs {
    /// Path to the submissionunit.xml file to validate
    #[arg(short, long)]
    pub file: PathBuf,
}

pub async fn execute(args: ValidateArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating: {:?}", args.file);

    // 1. Load File
    let xml_content = fs::read_to_string(&args.file)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // 2. Parse (Structural Check)
    let unit: SubmissionUnit = match from_str(&xml_content) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("❌ FATAL: XML Structure Violation");
            eprintln!("The file is not well-formed eCTD v4.0 XML.");
            eprintln!("Error: {}", e);
            return Ok(()); // Exit gracefully with error printed
        }
    };

    println!("✅ Structure OK. Running Compliance Rules...");

    // 3. Run the Validation Engine
    let validator = get_standard_validator();
    let errors = validator.run(&unit);

    // 4. Report Results
    if errors.is_empty() {
        println!("🎉 VALIDATION PASSED!");
        println!("No errors found. This submission is ready for ingestion.");
    } else {
        println!("⚠️  VALIDATION FAILED: Found {} errors.", errors.len());
        println!("{:-<50}", "-");

        for err in errors {
            // Color-coded output (conceptually)
            let icon = if err.severity.contains("High") { "🛑" } else { "⚠️" };
            println!("{} [{}] {}", icon, err.code, err.severity);
            println!("   Msg: {}", err.message);
            if let Some(target) = err.target_id {
                println!("   Ref: {}", target);
            }
            println!("{:-<50}", "-");
        }
    }

    Ok(())
}
