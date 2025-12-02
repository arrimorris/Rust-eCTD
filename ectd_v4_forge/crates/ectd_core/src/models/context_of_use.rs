use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// The Connector: Context of Use (CoU)
// Reference: PDF Section 4.2.5 "Context of Use"
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextOfUse {
    // Rule eCTD4-021: Must be a unique UUID
    #[serde(rename = "@id")]
    pub id: String,

    // Rule eCTD4-075: Code must be valid (e.g., matches a Keyword definition)
    #[serde(rename = "@code")]
    pub code: String,

    // Rule eCTD4-023: Status can only be "active" or "suspended"
    #[serde(rename = "@statusCode")]
    pub status_code: String,

    // Rule eCTD4-017: Priority Number is required
    // Rule eCTD4-018: Whole number between 1 and 999999
    #[serde(rename = "priorityNumber")]
    pub priority_number: u32,

    // Rule eCTD4-027: Document Reference required for active CoU
    #[serde(rename = "documentReference")]
    pub document_reference: Option<DocumentReference>,
}

// ---------------------------------------------------------------------------
// The Pointer: Document Reference
// Connects the CoU to the physical Document
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentReference {
    // Points to the UUID of the <document> element
    #[serde(rename = "id")]
    pub id: String,
}
