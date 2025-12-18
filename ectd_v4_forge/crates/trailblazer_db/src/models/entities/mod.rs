use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct SubmissionUnitEntity {
    pub id: Uuid,
    pub submission_id: Uuid,
    pub sequence_number: i32,
    pub code: String,
    pub code_system: String,
    pub status_code: String,

    // Application Info
    pub application_id_uuid: Option<Uuid>,
    pub application_code: Option<String>,
    pub application_number: Option<String>,
    pub applicant_name: Option<String>,
    pub submission_code: Option<String>,

    // XML Info
    pub xmlns: Option<String>,
    pub xmlns_xsi: Option<String>,
    pub schema_location: Option<String>,

    pub created_at: Option<time::OffsetDateTime>,
}

#[derive(Debug, FromRow)]
pub struct DocumentEntity {
    // ID is VARCHAR(255) in schema, not UUID
    pub id: String,
    pub submission_unit_id: Uuid,
    pub xlink_href: String,
    pub checksum: String,
    pub checksum_algorithm: String,
    pub title: Option<String>,
    pub media_type: String,
    pub created_at: Option<time::OffsetDateTime>,
}
