use tauri::{AppHandle, Emitter, State};
use ectd_service::{EctdService, documents::AddDocumentParams, submission::InitSubmissionParams};
use ectd_core::{get_standard_validator, models::submission_unit::*};
use uuid::Uuid;
use std::path::PathBuf;
use futures::StreamExt;
use crate::state::AppState;
use bollard::container::{InspectContainerOptions, StartContainerOptions};

// --- INPUT STRUCTS ---
#[derive(serde::Deserialize)]
pub struct InitArgs {
    pub app_number: String,
    pub app_type: String,
    pub applicant: String,
    pub sequence: u32,
}

// --- COMMANDS ---

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// SELF-HEALING: Checks if Docker/Postgres is running, starts it if not.
#[tauri::command]
pub async fn ensure_infrastructure(
    app_state: State<'_, AppState>,
    service: State<'_, EctdService>
) -> Result<String, String> {
    // 1. Try simple connectivity first (Fastest)
    if sqlx::query("SELECT 1").execute(&service.pool).await.is_ok() {
        return Ok("System Healthy (Connected)".to_string());
    }

    // 2. If DB failed, check Docker (Self-Healing)
    let docker = &app_state.docker;
    match docker.inspect_container("ectd_db", None::<InspectContainerOptions>).await {
        Ok(c) => {
            let running = c.state.and_then(|s| s.running).unwrap_or(false);
            if !running {
                // It exists but is stopped -> START IT
                docker.start_container("ectd_db", None::<StartContainerOptions<String>>).await.map_err(|e| e.to_string())?;

                // Wait a moment for Postgres to accept connections?
                // For MVP, just returning "Restarted" is fine; the next retry will connect.
                return Ok("Restarted Database Container. Please wait...".to_string());
            }
        },
        Err(_) => return Err("Critical: Database container 'ectd_db' is missing.".to_string()),
    }

    Err("Database is running but unreachable.".to_string())
}

#[tauri::command]
pub async fn init_submission(
    service: State<'_, EctdService>,
    args: InitArgs,
) -> Result<String, String> {
    // 1. Construct the Unit (Skeleton)
    let unit_id = Uuid::new_v4();
    let std_oid = "urn:oid:2.16.840.1.113883.3.989.2.2.1".to_string();

    let unit = SubmissionUnit {
        id: unit_id.to_string(),
        code: "original-application".to_string(),
        code_system: std_oid.clone(),
        status_code: "active".to_string(),
        xmlns: "urn:hl7-org:v3".to_string(),
        xmlns_xsi: Some("http://www.w3.org/2001/XMLSchema-instance".to_string()),
        schema_location: None,

        submission: Submission {
            id: Uuid::new_v4().to_string(),
            code: format!("seq-{:04}", args.sequence),
            code_system: std_oid.clone(),
            sequence_number: SequenceNumber { value: args.sequence },
        },
        application: Application {
            id: Uuid::new_v4().to_string(),
            code: args.app_type,
            code_system: std_oid.clone(),
            application_number: ApplicationNumber { code: args.app_number, code_system: std_oid.clone() },
        },
        applicant: Applicant {
            sponsoring_organization: SponsoringOrganization { name: args.applicant },
        },
        context_of_use: vec![],
        documents: vec![],
        keyword_definitions: Some(vec![]),
    };

    // 2. Persist
    let repo = ectd_db::repository::SubmissionRepository::new(service.pool.clone());
    repo.create_submission(&unit).await.map_err(|e| e.to_string())?;

    Ok(unit_id.to_string())
}

#[tauri::command]
pub async fn add_document(
    service: State<'_, EctdService>,
    submission_id: String,
    file_path: String,
    context: String,
    title: String,
) -> Result<String, String> {
    let sub_uuid = Uuid::parse_str(&submission_id).map_err(|e| e.to_string())?;

    let params = AddDocumentParams {
        submission_id: sub_uuid,
        file_path: PathBuf::from(file_path),
        context_code: context,
        title: title,
        priority: 1,
    };

    let doc_id = service.attach_document(params).await.map_err(|e| e.to_string())?;
    Ok(doc_id.to_string())
}

#[tauri::command]
pub async fn validate_submission(
    service: State<'_, EctdService>,
    submission_id: String,
) -> Result<Vec<String>, String> {
    let uuid = Uuid::parse_str(&submission_id).map_err(|e| e.to_string())?;

    let repo = ectd_db::repository::SubmissionRepository::new(service.pool.clone());
    let unit = repo.get_submission(uuid).await.map_err(|e| e.to_string())?;

    let validator = get_standard_validator();
    let errors = validator.run(&unit);

    let report = errors.into_iter()
        .map(|e| format!("[{}] {}: {}", e.severity, e.code, e.message))
        .collect();

    Ok(report)
}

#[tauri::command]
pub async fn export_submission(
    app: AppHandle, // <--- Event Emitter
    service: State<'_, EctdService>,
    submission_id: String,
    target_dir: String,
) -> Result<String, String> {
    let uuid = Uuid::parse_str(&submission_id).map_err(|e| e.to_string())?;
    let path = PathBuf::from(target_dir);

    // Use the STREAMING service method
    let mut stream = service.export_submission_stream(uuid, path);
    // Removed pin_mut! because we now use Box::pin in the service which returns Pin<Box<...>>

    // Forward events to Frontend
    while let Some(progress_result) = stream.next().await {
        match progress_result {
            Ok(progress) => {
                app.emit("export-progress", progress).map_err(|e| e.to_string())?;
            },
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok("Export Successful".to_string())
}
