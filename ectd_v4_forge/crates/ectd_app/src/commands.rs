use tauri::{AppHandle, Emitter, State};
use ectd_service::{EctdService, documents::AddDocumentParams, submission::InitSubmissionParams};
use ectd_core::{get_standard_validator, models::submission_unit::*};
use uuid::Uuid;
use std::path::PathBuf;
use futures::StreamExt;
// Removed pin_mut import
use crate::state::AppState;
use bollard::container::{InspectContainerOptions, StartContainerOptions};

// Basic test command
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Define a struct for the Init payload from frontend
#[derive(serde::Deserialize)]
pub struct InitArgs {
    pub app_number: String,
    pub app_type: String,
    pub applicant: String,
    pub sequence: u32,
}

#[tauri::command]
pub async fn init_submission(
    service: State<'_, EctdService>,
    args: InitArgs,
) -> Result<String, String> {
    let params = InitSubmissionParams {
        app_number: args.app_number,
        app_type: args.app_type,
        applicant_name: args.applicant,
        sequence_number: args.sequence,
        submission_code: format!("seq-{:04}", args.sequence),
    };

    let unit_id = service.create_submission(params).await
        .map_err(|e| e.to_string())?;

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

    // 1. Fetch from DB
    let repo = ectd_db::repository::SubmissionRepository::new(service.pool.clone());
    let unit = repo.get_submission(uuid).await.map_err(|e| e.to_string())?;

    // 2. Run Validation Engine
    let validator = get_standard_validator();
    let errors = validator.run(&unit);

    // 3. Format Errors for UI
    let report = errors.into_iter()
        .map(|e| format!("[{}] {}: {}", e.severity, e.code, e.message))
        .collect();

    Ok(report)
}

#[tauri::command]
pub async fn export_submission(
    app: AppHandle,
    service: State<'_, EctdService>,
    submission_id: String,
    target_dir: String,
) -> Result<String, String> {
    let uuid = Uuid::parse_str(&submission_id).map_err(|e| e.to_string())?;
    let path = PathBuf::from(target_dir);

    let mut stream = service.export_submission_stream(uuid, path);
    // Removed pin_mut!

    while let Some(progress_result) = stream.next().await {
        match progress_result {
            Ok(progress) => {
                app.emit("export-progress", progress).map_err(|e| e.to_string())?;
            },
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok("Done".to_string())
}

#[tauri::command]
pub async fn ensure_infrastructure(state: State<'_, AppState>) -> Result<String, String> {
    let docker = &state.docker;

    // Check if DB is running
    match docker.inspect_container("ectd_db", None::<InspectContainerOptions>).await {
        Ok(c) => {
            if !c.state.unwrap().running.unwrap_or(false) {
                // It exists but is stopped -> START IT
                docker.start_container("ectd_db", None::<StartContainerOptions<String>>).await.map_err(|e| e.to_string())?;
                return Ok("Restarted Database".to_string());
            }
        },
        Err(_) => return Err("Database container missing! Run docker-compose up.".to_string()),
    }

    Ok("System Healthy".to_string())
}
