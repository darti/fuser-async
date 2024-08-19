use std::time::SystemTime;

use serde_with::DefaultOnNull;
use serde_with::{serde::Deserialize, serde_as};

#[serde_as]
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RmkMetadata {
    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(default)]
    pub _deleted: bool,
    // #[serde_as(as = "serde_with::TimestampSeconds<String>")]
    // last_opened: SystemTime,
    // last_opened_page: usize,
    #[serde_as(as = "serde_with::TimestampSeconds<String>")]
    pub last_modified: SystemTime,

    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(default)]
    pub _metadatamodified: bool,

    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(default)]
    pub _modified: bool,

    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub parent: Option<String>,
    pub _pinned: bool,

    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(default)]
    pub _synced: bool,

    #[serde(rename = "type")]
    pub typ: String,

    #[serde_as(deserialize_as = "DefaultOnNull")]
    #[serde(default)]
    pub _version: usize,

    pub visible_name: String,
}

//     "createdTime": "1707124703715",
//     "lastModified": "1707304046127",
//     "lastOpened": "1707296430499",
//     "lastOpenedPage": 1,
//     "parent": "37d53cbe-e82d-4969-86fe-e5bc365a9f1f",
//     "pinned": false,
//     "type": "DocumentType",
//     "visibleName": "Amundi tomorrow"
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_metadata() {
        let data = json!({
                "createdTime": "1707124703715",
                "lastModified": "1707304046127",
                "lastOpened": "1707296430499",
                "lastOpenedPage": 1,
                "parent": "37d53cbe-e82d-4969-86fe-e5bc365a9f1f",
                "pinned": false,
                "type": "DocumentType",
                "visibleName": "Amundi tomorrow"
        });

        let metadata: RmkMetadata = serde_json::from_value(data).unwrap();

        assert_eq!(metadata._deleted, false);
        assert_eq!(metadata._metadatamodified, false);
        assert_eq!(metadata._modified, false);
        assert_eq!(
            metadata.parent.unwrap(),
            "37d53cbe-e82d-4969-86fe-e5bc365a9f1f"
        );
        assert_eq!(metadata._pinned, false);
        assert_eq!(metadata._synced, false);
        assert_eq!(metadata.typ, "DocumentType");
        assert_eq!(metadata._version, 0);
        assert_eq!(metadata.visible_name, "Amundi tomorrow");
    }
}
