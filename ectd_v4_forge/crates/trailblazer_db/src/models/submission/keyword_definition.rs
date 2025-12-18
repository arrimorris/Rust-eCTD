use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct KeywordDefinition {
    pub submission_unit_id: Uuid,
    pub code: String,
    pub code_system: String,
    pub display_name: String,
}
