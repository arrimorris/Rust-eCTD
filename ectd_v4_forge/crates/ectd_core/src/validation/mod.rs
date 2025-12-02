use crate::models::submission_unit::SubmissionUnit;

pub trait ValidationRule {
    fn check(&self, unit: &SubmissionUnit) -> Result<(), ValidationError>;
    fn error_code(&self) -> String;
}

#[derive(Debug)]
pub struct ValidationError {
    pub code: String,
    pub severity: String,
    pub message: String,
}

// Example Implementation of Rule eCTD4-004 from your PDF
// "Submission Unit id root must be a UUID"
pub struct CheckSubmissionUnitUUID;

impl ValidationRule for CheckSubmissionUnitUUID {
    fn error_code(&self) -> String {
        "eCTD4-004".to_string()
    }

    fn check(&self, unit: &SubmissionUnit) -> Result<(), ValidationError> {
        // Use the uuid crate to parse the string
        match uuid::Uuid::parse_str(&unit.id) {
            Ok(_) => Ok(()),
            Err(_) => Err(ValidationError {
                code: self.error_code(),
                severity: "High Error".to_string(), // From PDF source [50]
                message: "Submission Unit id root must be a UUID".to_string(),
            }),
        }
    }
}
