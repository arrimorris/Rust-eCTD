use super::{ValidationRule, ValidationError, ValidationSeverity};
use trailblazer_db::models::submission::{
    submission_unit::SubmissionUnit,
    document::Document,
    context_of_use::ContextOfUse,
};

// =========================================================================
// RULE: eCTD4-013 (Sequence Number)
// =========================================================================
pub struct SequenceNumberRule;

impl ValidationRule for SequenceNumberRule {
    fn id(&self) -> &'static str { "eCTD4-013" }

    fn check(
        &self,
        unit: &SubmissionUnit,
        _docs: &[Document],
        _ctx: &[ContextOfUse]
    ) -> Vec<ValidationError> {
        let mut errors = vec![];

        // Correctness Check: eCTD v4.0 requires 4 digits (0001-9999).
        // Since we use i32, we must strictly enforce the bounds.
        if unit.sequence_number < 1 || unit.sequence_number > 9999 {
            errors.push(ValidationError {
                rule_id: self.id().to_string(),
                description: format!(
                    "Sequence number {} is out of valid range (0001-9999).",
                    unit.sequence_number
                ),
                severity: ValidationSeverity::Error,
            });
        }
        errors
    }
}

// =========================================================================
// RULE: eCTD4-048 (Document Integrity)
// =========================================================================
pub struct DocumentIntegrityRule;

impl ValidationRule for DocumentIntegrityRule {
    fn id(&self) -> &'static str { "eCTD4-048" }

    fn check(
        &self,
        _unit: &SubmissionUnit,
        docs: &[Document],
        _ctx: &[ContextOfUse]
    ) -> Vec<ValidationError> {
        let mut errors = vec![];

        for doc in docs {
            // Correctness Check: FDA requires SHA-256.
            // In the DB, this is stored as a hex string. length must be 64 chars.
            if doc.checksum_algorithm == "SHA-256" && doc.checksum.len() != 64 {
                errors.push(ValidationError {
                    rule_id: self.id().to_string(),
                    description: format!(
                        "Document '{}' has invalid SHA-256 checksum length (Found {}, expected 64).",
                        doc.title, doc.checksum.len()
                    ),
                    severity: ValidationSeverity::Error,
                });
            }

            // TODO: Add check for empty xlink_href or missing physical file
        }
        errors
    }
}

// =========================================================================
// The Registry
// =========================================================================
pub fn get_all_rules() -> Vec<Box<dyn ValidationRule + Send + Sync>> {
    vec![
        Box::new(SequenceNumberRule),
        Box::new(DocumentIntegrityRule),
    ]
}
