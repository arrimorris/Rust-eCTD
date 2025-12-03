use serde::{Deserialize, Serialize};
use quick_xml::se::to_string;
use anyhow::Result;

// ---------------------------------------------------------------------------
// 1. The Root Container: <submissionUnit>
// Reference: PDF Section 4.2.2
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")] // eCTD v4.0 tags are usually camelCase
pub struct SubmissionUnit {
    // -------------------
    // Root Attributes
    // -------------------
    #[serde(rename = "@xmlns")]
    pub xmlns: String, // usually "urn:hl7-org:v3"

    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: Option<String>,

    #[serde(rename = "@xsi:schemaLocation")]
    pub schema_location: Option<String>,

    // -------------------
    // Submission Unit Metadata
    // -------------------
    // Rule eCTD4-003: Identifier is required
    // Rule eCTD4-004: Must be a UUID
    #[serde(rename = "@id")]
    pub id: String,

    // Note: The previous "submission_id" and "sequence_number" fields are now nested
    // inside the 'submission' block in the v4.0 spec, but our Repository logic expects them.
    // For now, I will keep the struct aligned with the XML spec provided by Gemini.
    // I will need to update the Repository to extract these from the nested structs.

    // Rule eCTD4-006: Code value is required
    #[serde(rename = "@code")]
    pub code: String, // e.g., "original-application"

    // Rule eCTD4-008: Code System OID
    #[serde(rename = "@codeSystem")]
    pub code_system: String,

    // Rule eCTD4-010: Status must be "active"
    #[serde(rename = "@statusCode")]
    pub status_code: String,

    // -------------------
    // The "Big 3" Metadata Blocks
    // -------------------
    // Reference: PDF 4.2.9
    pub submission: Submission,

    // Reference: PDF 4.2.10
    pub application: Application,

    // Reference: PDF 4.2.11
    pub applicant: Applicant,

    // -------------------
    // The Content Graph
    // -------------------
    // Rule eCTD4-011: At least one CoU required
    #[serde(rename = "contextOfUse", default)]
    pub context_of_use: Vec<ContextOfUse>,

    // Reference: PDF 4.2.13
    #[serde(rename = "document", default)]
    pub documents: Vec<Document>,

    // Reference: PDF 4.2.14
    #[serde(rename = "keywordDefinition", default)]
    pub keyword_definitions: Option<Vec<KeywordDefinition>>,
}

impl SubmissionUnit {
    /// Serializes the struct to a canonical eCTD v4.0 XML string
    /// Includes the correct XML declaration and encoding.
    pub fn to_xml(&self) -> Result<String> {
        let xml_body = to_string(&self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize SubmissionUnit: {}", e))?;

        // The eCTD standard requires UTF-8 and version 1.0
        Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, xml_body))
    }
}

// ---------------------------------------------------------------------------
// 2. Metadata Blocks (Submission, Application, Applicant)
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    #[serde(rename = "@id")]
    pub id: String, // UUIDv7

    #[serde(rename = "@code")]
    pub code: String, // e.g., "seq-0001"

    #[serde(rename = "@codeSystem")]
    pub code_system: String,

    // Rule eCTD4-013: Sequence Number
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: SequenceNumber,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceNumber {
    #[serde(rename = "@value")]
    pub value: u32, // 0001
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    #[serde(rename = "@id")]
    pub id: String, // UUIDv7

    #[serde(rename = "@code")]
    pub code: String, // Application Type (e.g., "nda")

    #[serde(rename = "@codeSystem")]
    pub code_system: String,

    #[serde(rename = "code")]
    pub application_number: ApplicationNumber,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationNumber {
    // Rule US-eCTD4-510: 6 digits only
    #[serde(rename = "@code")]
    pub code: String,

    #[serde(rename = "@codeSystem")]
    pub code_system: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Applicant {
    pub sponsoring_organization: SponsoringOrganization,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsoringOrganization {
    #[serde(rename = "name")]
    pub name: String,
}

// Re-exports for other modules
pub use crate::models::context_of_use::ContextOfUse;
pub use crate::models::document::Document;
pub use crate::models::keyword_definition::KeywordDefinition;
