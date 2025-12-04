use clap::Args;
use sqlx::PgPool;
use uuid::Uuid;
use ectd_core::models::submission_unit::*;
use ectd_db::repository::SubmissionRepository;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Application Number (e.g. 123456)
    #[arg(long)]
    pub app_number: String,

    /// Application Type (nda, ind, bla)
    #[arg(long, default_value = "nda")]
    pub app_type: String,

    /// Applicant Name (e.g. "Acme Pharmaceuticals")
    #[arg(long)]
    pub applicant: String,

    /// Sequence Number (e.g. 1)
    #[arg(long, default_value_t = 1)]
    pub sequence: u32,
}

pub async fn execute(pool: PgPool, args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Initializing New Submission...");
    println!("   Applicant: {}", args.applicant);
    println!("   App #:     {} ({})", args.app_number, args.app_type);
    println!("   Sequence:  {:04}", args.sequence);

    // 1. Construct the Skeleton (Pure Rust Logic)
    let unit_id = Uuid::new_v4();
    let submission_id = Uuid::new_v4();
    let app_id = Uuid::new_v4();

    // Standard OID for FDA/ICH v4.0
    let std_oid = "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string();

    let unit = SubmissionUnit {
        xmlns: "urn:hl7-org:v3".to_string(),
        xmlns_xsi: Some("http://www.w3.org/2001/XMLSchema-instance".to_string()),
        schema_location: Some("urn:hl7-org:v3 ../../schema/rps.xsd".to_string()),
        id: unit_id.to_string(),
        code: "original-application".to_string(), // Default code
        code_system: std_oid.clone(),
        status_code: "active".to_string(),

        submission: Submission {
            id: submission_id.to_string(),
            code: format!("seq-{:04}", args.sequence),
            code_system: std_oid.clone(),
            sequence_number: SequenceNumber { value: args.sequence },
        },

        application: Application {
            id: app_id.to_string(),
            code: args.app_type,
            code_system: std_oid.clone(),
            application_number: ApplicationNumber {
                code: args.app_number,
                code_system: std_oid.clone(),
            },
        },

        applicant: Applicant {
            sponsoring_organization: SponsoringOrganization {
                name: args.applicant,
            },
        },

        // Empty containers for now
        context_of_use: vec![],
        documents: vec![],
        keyword_definitions: Some(vec![]),
    };

    // 2. Persist to DB
    let repo = SubmissionRepository::new(pool);
    repo.create_submission(&unit).await?;

    println!("✅ Submission Initialized successfully.");
    println!("🔑 UUID: {}", unit_id);
    println!("📝 Next: Use 'add-doc' (coming soon) to populate this submission.");

    Ok(())
}
