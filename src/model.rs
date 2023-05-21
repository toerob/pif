use serde::Deserialize;

#[derive(Deserialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    pub extensions: Vec<Extension>,
}

#[derive(Deserialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    pub name: String,
    pub author: Option<String>,
    pub desc: Option<String>,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    #[serde(rename = "type")]
    pub extension_type: Option<Vec<String>>,

    pub version: Option<String>,

    pub url: Option<String>,

    #[serde(rename = "makefile-entries")]
    pub makefile_entries: Option<Vec<String>>,
    pub ext: Option<String>,

    #[serde(rename = "last-modified")]
    pub last_modified: Option<String>,
}
