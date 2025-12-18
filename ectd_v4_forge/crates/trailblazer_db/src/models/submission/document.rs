use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Document {
    pub id: Uuid,
    pub submission_unit_id: Uuid,

    // Physical Reference
    pub xlink_href: String,
    pub media_type: String,

    // Integrity
    pub checksum: String,
    pub checksum_algorithm: String,

    // Metadata
    pub title: String,
}
