use std::fmt::Display;

use serde::Deserialize;
use serde::Serialize;


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

    #[serde(deserialize_with = "deserialize_version")]
    pub version: semver::Version,

    pub url: Option<String>,

    #[serde(rename = "makefile-entries")]
    pub makefile_entries: Option<Vec<String>>,
    pub ext: Option<String>,
    pub branch: Option<String>,

    #[serde(rename = "last-modified")]
    pub last_modified: Option<String>,
}


// Anpassad deserialisering
fn deserialize_version<'de, D>(deserializer: D) -> Result<semver::Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version_str: String = String::deserialize(deserializer)?;
    let normalized = if version_str.eq_ignore_ascii_case("SNAPSHOT") {
        "0.0.0-SNAPSHOT".to_string() // Normalisera SNAPSHOT        
    } else if version_str.matches('.').count() == 1 {
        format!("{}.0", version_str) // Lägg till patch
    } else {
        version_str
    };
    print!("NORMALIZED TO: {}\n", normalized);

    semver::Version::parse(&normalized).map_err(serde::de::Error::custom)
}

