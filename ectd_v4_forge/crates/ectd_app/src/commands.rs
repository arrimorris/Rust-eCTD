use tauri::State;
use ectd_service::{EctdService, documents::AddDocumentParams, submission::InitSubmissionParams};
use uuid::Uuid;
use std::path::PathBuf;

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
