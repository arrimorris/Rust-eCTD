use tauri::State;
use ectd_service::{EctdService, submission::InitSubmissionParams};

// Basic test command
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Wrapper for the 'Init' logic
#[tauri::command]
pub async fn init_submission(
    service: State<'_, EctdService>,
    app_number: String,
    applicant: String,
    sequence: u32,
    submission_code: String,
) -> Result<String, String> {
    let params = InitSubmissionParams {
        app_number,
        applicant_name: applicant,
        sequence_number: sequence,
        submission_code,
    };

    let unit_id = service.create_submission(params).await
        .map_err(|e| e.to_string())?;

    Ok(format!("Submission Created! UUID: {}", unit_id))
}
