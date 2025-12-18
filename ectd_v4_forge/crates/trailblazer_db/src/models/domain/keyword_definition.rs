use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 6. Keyword Definitions (Custom Vocabulary)
// Reference: PDF Section 4.2.14
// ---------------------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordDefinition {
    #[serde(rename = "@code")]
    pub code: String,

    #[serde(rename = "@codeSystem")]
    pub code_system: String,

    #[serde(rename = "value")]
    pub value: KeywordDefinitionValue,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordDefinitionValue {
    #[serde(rename = "item")]
    pub item: KeywordDefinitionItem,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeywordDefinitionItem {
    #[serde(rename = "@code")]
    pub code: String,

    #[serde(rename = "displayName")]
    pub display_name: DisplayName,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayName {
    #[serde(rename = "@value")]
    pub value: String,
}
