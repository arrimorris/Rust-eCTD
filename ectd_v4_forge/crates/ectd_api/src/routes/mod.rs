use axum::{routing::get, Router};
use crate::handlers::health_check;

pub fn app_router() -> Router {
    Router::new().route("/health", get(health_check))
}
