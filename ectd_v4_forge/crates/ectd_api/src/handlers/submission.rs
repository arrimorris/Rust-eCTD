use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use ectd_core::models::submission_unit::SubmissionUnit;
use ectd_db::repository::SubmissionRepository;
use crate::AppState;

pub async fn get_submission(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SubmissionUnit>, (StatusCode, String)> {
    let repo = SubmissionRepository::new(state.pool);

    match repo.get_submission(id).await {
        Ok(unit) => Ok(Json(unit)),
        Err(sqlx::Error::RowNotFound) => {
            Err((
                StatusCode::NOT_FOUND,
                format!("Submission Unit not found: {}", id),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to fetch submission {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal server error"),
            ))
        }
    }
}
