use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ContextOfUse {
    pub id: Uuid,
    pub submission_unit_id: Uuid,

    // Classification
    pub code: String,
    pub code_system: String,

    // Lifecycle
    pub status_code: String,
    pub priority_number: i32,

    // Links
    pub document_reference_id: Option<Uuid>,
    pub replaces_context_id: Option<Uuid>,
}
