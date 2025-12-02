use crate::models::submission_unit::SubmissionUnit;
use crate::validation::{ValidationError, ValidationRule};
use uuid::Uuid;

// =========================================================================
// RULE: eCTD4-004
// "Submission Unit id root must be a UUID"
// Source: PDF Section 4.2.2
// =========================================================================
pub struct RuleEctd4_004;

impl ValidationRule for RuleEctd4_004 {
    fn rule_id(&self) -> &str { "eCTD4-004" }

    fn check(&self, unit: &SubmissionUnit) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        if Uuid::parse_str(&unit.id).is_err() {
            errors.push(ValidationError {
                code: self.rule_id().to_string(),
                severity: "High Error".to_string(),
                message: format!("Submission Unit ID '{}' is not a valid UUID", unit.id),
                target_id: Some(unit.id.clone()),
            });
        }
        errors
    }
}

// =========================================================================
// RULE: eCTD4-006
// "Submission Unit code value is required"
// Source: PDF Section 4.2.2
// =========================================================================
pub struct RuleEctd4_006;

impl ValidationRule for RuleEctd4_006 {
    fn rule_id(&self) -> &str { "eCTD4-006" }

    fn check(&self, unit: &SubmissionUnit) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        if unit.code.trim().is_empty() {
            errors.push(ValidationError {
                code: self.rule_id().to_string(),
                severity: "High Error".to_string(),
                message: "Submission Unit code attribute is required and cannot be empty".to_string(),
                target_id: Some(unit.id.clone()),
            });
        }
        errors
    }
}

// =========================================================================
// RULE: eCTD4-013
// "Sequence Number must be a whole number between 1 and 999999"
// Source: PDF Section 4.2.3
// =========================================================================
pub struct RuleEctd4_013;

impl ValidationRule for RuleEctd4_013 {
    fn rule_id(&self) -> &str { "eCTD4-013" }

    fn check(&self, unit: &SubmissionUnit) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let seq = unit.submission.sequence_number.value;

        if seq < 1 || seq > 999999 {
            errors.push(ValidationError {
                code: self.rule_id().to_string(),
                severity: "High Error".to_string(),
                message: format!("Sequence Number '{}' is invalid. Must be between 1 and 999999.", seq),
                target_id: Some(unit.submission.id.clone()),
            });
        }
        errors
    }
}

// =========================================================================
// RULE: eCTD4-048
// "Document text element requires a checksum value"
// Source: PDF Section 4.2.13
// =========================================================================
pub struct RuleEctd4_048;

impl ValidationRule for RuleEctd4_048 {
    fn rule_id(&self) -> &str { "eCTD4-048" }

    fn check(&self, unit: &SubmissionUnit) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for doc in &unit.documents {
            if doc.text.checksum.trim().is_empty() {
                errors.push(ValidationError {
                    code: self.rule_id().to_string(),
                    severity: "High Error".to_string(),
                    message: "Document missing checksum value".to_string(),
                    target_id: Some(doc.id.clone()),
                });
            }
        }
        errors
    }
}

// TODO: Implement remaining rules from eCTD v4.0 Specification (Section 4.2)
