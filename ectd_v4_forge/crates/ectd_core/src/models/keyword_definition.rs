use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// The Vocabulary: Keyword Definition
// Reference: PDF Section 4.2.14 "Keyword Definition"
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordDefinition {
    // Rule eCTD4-052: Code is required
    #[serde(rename = "@code")]
    pub code: String,

    // Rule eCTD4-056: Value is required
    #[serde(rename = "value")]
    pub value: KeywordValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordValue {
     #[serde(rename = "item")]
     pub item: KeywordItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordItem {
     #[serde(rename = "@code")]
     pub code: String,

     #[serde(rename = "displayName")]
     pub display_name: String,
}
