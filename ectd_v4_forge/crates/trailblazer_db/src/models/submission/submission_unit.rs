use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct SubmissionUnit {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub sequence_number: i32,

    // Metadata
    pub code: String,
    pub code_system: String,
    pub status_code: String,

    // Audit
    #[serde(with = "time::serde::iso8601")]
    pub created_at: OffsetDateTime,
}

impl SubmissionUnit {
    pub fn new(submission_id: Uuid, sequence_number: i32) -> Self {
        Self {
            id: Uuid::now_v7(),
            submission_id,
            sequence_number,
            code: "submission-unit".to_string(), // Default eCTD code
            code_system: "2.16.840.1.113883.3.989.2.1.1.1".to_string(), // Default OID
            status_code: "active".to_string(),
            created_at: OffsetDateTime::now_utc(),
        }
    }
}
