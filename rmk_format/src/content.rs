use serde_with::{serde::Deserialize, serde_as};

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RmkContent {
    pub file_type: Option<String>,
    pub page_count: usize,
    pub pages: Vec<String>,
    pub orientation: String,
}
