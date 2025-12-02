use serde::{Deserialize, Serialize};
use crate::models::context_of_use::ContextOfUse;
use crate::models::document::Document;
use crate::models::keyword_definition::KeywordDefinition;

// ---------------------------------------------------------------------------
// The Root: SubmissionUnit (v4.0 Style)
// Reference: PDF Section 4.2.2 "Submission Unit"
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "submissionUnit")]
pub struct SubmissionUnit {
    // Rule eCTD4-003: Identifier is required
    // Rule eCTD4-004: Must be a UUID
    #[serde(rename = "@id")]
    pub id: String,

    // Rule eCTD4-006: Code value is required (e.g., "original-application")
    #[serde(rename = "@code")]
    pub code: String,

    // Rule eCTD4-008: Code System OID is required
    #[serde(rename = "@codeSystem")]
    pub code_system: String,

    // Rule eCTD4-011: Must have at least one Context of Use
    #[serde(rename = "contextOfUse")]
    pub context_of_use: Vec<ContextOfUse>,

    #[serde(rename = "document")]
    pub documents: Vec<Document>,

    #[serde(rename = "keywordDefinition")]
    pub keyword_definitions: Option<Vec<KeywordDefinition>>,
}
