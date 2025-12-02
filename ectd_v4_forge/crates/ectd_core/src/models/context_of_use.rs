use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 3. The Connector: Context of Use (CoU)
// Reference: PDF Section 4.2.5
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextOfUse {
    #[serde(rename = "@id")]
    pub id: String, // UUID

    #[serde(rename = "@code")]
    pub code: String, // What is this? (e.g. "cover-letter")

    #[serde(rename = "@codeSystem")]
    pub code_system: String,

    #[serde(rename = "@statusCode")]
    pub status_code: String, // "active" or "suspended"

    // Rule eCTD4-017: Priority Number
    #[serde(rename = "priorityNumber")]
    pub priority_number: PriorityNumber,

    // Link to the physical document
    // Rule eCTD4-027: Required for active CoU
    #[serde(rename = "documentReference")]
    pub document_reference: Option<DocumentReference>,

    // Lifecycle: Replacing an old CoU?
    #[serde(rename = "relatedContextOfUse", default)]
    pub related_context_of_use: Option<RelatedContextOfUse>,

    // Keywords attached to this CoU
    #[serde(rename = "keyword", default)]
    pub keywords: Vec<Keyword>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriorityNumber {
    #[serde(rename = "@value")]
    pub value: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentReference {
    #[serde(rename = "id")]
    pub id: DocumentIdRef,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentIdRef {
    #[serde(rename = "@root")]
    pub root: String, // The UUID of the <document> element
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedContextOfUse {
    #[serde(rename = "id")]
    pub id: DocumentIdRef, // Points to the PREVIOUS CoU UUID

    #[serde(rename = "@relationshipName")]
    pub relationship_name: String, // "replaces"
}

// ---------------------------------------------------------------------------
// 4. The Keywords
// Reference: PDF Section 4.2.8
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct Keyword {
    #[serde(rename = "@code")]
    pub code: String,

    #[serde(rename = "@codeSystem")]
    pub code_system: String,
}
