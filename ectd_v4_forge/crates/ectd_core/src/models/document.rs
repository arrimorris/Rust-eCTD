use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// The Payload: Document
// Reference: PDF Section 4.2.13 "Document"
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    // Rule eCTD4-045: Must be a UUID
    #[serde(rename = "@id")]
    pub id: String,

    // Rule eCTD4-050: Document path is required
    #[serde(rename = "@xlink:href")]
    pub href: String,

    // Rule eCTD4-048: Checksum is required (SHA-256 for v4.0)
    #[serde(rename = "@integrityCheck")]
    pub checksum: String,

    #[serde(rename = "@integrityCheckAlgorithm")]
    pub checksum_algorithm: String, // "SHA-256"

    #[serde(rename = "title")]
    pub title: String,
}
