use axum::{routing::get, Router};
use crate::{handlers::{health_check, submission}, AppState};

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/submissions/:id", get(submission::get_submission))
        .with_state(state)
}
