use uuid::Uuid;
use time::OffsetDateTime;
use trailblazer_db::models::submission::submission_unit::SubmissionUnit;

pub struct SubmissionFactory;

impl SubmissionFactory {
    /// Creates a pristine, strictly compliant SubmissionUnit.
    ///
    /// This function ensures that any Unit born into the system starts with
    /// valid default OIDs and status codes, preventing "Garbage In".
    pub fn create_initial_submission(
        submission_id: Uuid,
        sequence_number: i32
    ) -> SubmissionUnit {
        SubmissionUnit {
            id: Uuid::now_v7(),
            submission_id,
            sequence_number,

            // Correctness: Hardcoded defaults for eCTD v4.0.
            // These OIDs are mandated by the ICH Implementation Guide.
            code: "submission-unit".to_string(),
            code_system: "2.16.840.1.113883.3.989.2.1.1.1".to_string(),
            status_code: "active".to_string(),

            created_at: OffsetDateTime::now_utc(),
        }
    }
}
