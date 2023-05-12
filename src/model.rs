
use serde::{Deserialize, Serialize};

#[derive(Deserialize,PartialOrd,Ord, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    #[serde(rename = "type")]
    pub extension_type: Option<Vec<String>>,

    pub name: String,
    pub author: Option<String>,
    pub desc: Option<String>,
    pub url: Option<String>,

    #[serde(rename = "makefile-entries")]
    pub makefile_entries: Option<Vec<String>>,
    pub ext: Option<String>,
    pub version: Option<String>,

    #[serde(rename = "last-modified")]
    pub last_modified: Option<String>,
}

#[derive(Deserialize,PartialOrd,Ord, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    pub extensions: Vec<Extension>,
}
