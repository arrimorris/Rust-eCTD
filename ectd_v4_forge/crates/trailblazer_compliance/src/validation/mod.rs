use serde::Serialize;
use trailblazer_db::models::submission::{
    submission_unit::SubmissionUnit,
    document::Document,
    context_of_use::ContextOfUse,
};

#[derive(Debug, Serialize, Clone)]
pub struct ValidationError {
    pub rule_id: String,
    pub description: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Serialize, Clone)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// The Trailblazer Validation Contract
///
/// Unlike the legacy system which validated a nested object tree,
/// this contract validates the "relational slice" directly from the Bedrock.
pub trait ValidationRule {
    fn check(
        &self,
        unit: &SubmissionUnit,
        documents: &[Document],
        contexts: &[ContextOfUse]
    ) -> Vec<ValidationError>;

    /// The FDA/ICH Rule ID (e.g., "eCTD4-013")
    fn id(&self) -> &'static str;
}

pub mod rules;
// TODO: Port PDF rules to new flat architecture.
// We comment this out to ensure we do not regress correctness by rushing a bad port.
// pub mod rules_pdf;
