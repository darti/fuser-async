use std::time::SystemTime;

use serde_with::{serde::Deserialize, serde_as};

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RmkMetadata {
    pub _deleted: bool,
    // #[serde_as(as = "serde_with::TimestampSeconds<String>")]
    // last_opened: SystemTime,
    // last_opened_page: usize,
    #[serde_as(as = "serde_with::TimestampSeconds<String>")]
    pub last_modified: SystemTime,
    pub _metadatamodified: bool,
    pub _modified: bool,

    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub parent: Option<String>,
    pub _pinned: bool,
    pub _synced: bool,
    #[serde(rename = "type")]
    pub typ: String,
    pub _version: usize,
    pub visible_name: String,
}
