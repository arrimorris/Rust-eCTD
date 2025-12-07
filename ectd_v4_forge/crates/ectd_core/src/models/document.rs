use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 5. The Physical Document
// Reference: PDF Section 4.2.13
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    #[serde(rename = "@id")]
    pub id: String, // UUID

    // The Title (displayed in the tree)
    #[serde(rename = "title")]
    pub title: DocumentTitle,

    // The Physical File Reference
    #[serde(rename = "text")]
    pub text: DocumentText,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentTitle {
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentText {
    // Rule eCTD4-050: Document Path
    #[serde(rename = "reference")]
    pub reference: DocumentReferencePath,

    // Rule eCTD4-048: Checksum
    #[serde(rename = "@integrityCheck")]
    pub checksum: String,

    #[serde(rename = "@integrityCheckAlgorithm")]
    pub checksum_algorithm: String, // "SHA256"

    #[serde(rename = "@mediaType")]
    pub media_type: String, // "application/pdf"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DocumentReferencePath {
    #[serde(rename = "@value")]
    pub value: String, // "m1/us/cover.pdf"
}
