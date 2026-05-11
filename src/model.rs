use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Extensions {
    #[serde(rename = "schema-version", skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    pub extensions: Vec<Extension>,
}

impl Extensions {
    /// Returns a list of human-readable warnings for malformed entries.
    /// Call this after deserialization and print any warnings to the user.
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        for ext in &self.extensions {
            for (i, v) in ext.versions.iter().enumerate() {
                if v.branch.is_some() && v.ext.is_some() {
                    warnings.push(format!(
                        "{} version {} (index {}): has both `branch` (git) and `ext` (archive) — \
                         these are mutually exclusive. `ext` will be ignored.",
                        ext.name,
                        v.version.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "unknown".into()),
                        i
                    ));
                }
                if v.url.is_none() {
                    warnings.push(format!(
                        "{} version {} (index {}): missing `url`.",
                        ext.name,
                        v.version.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "unknown".into()),
                        i
                    ));
                }
            }
        }
        warnings
    }
}

#[derive(Deserialize, Serialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    // last-modified first so it appears at the top of each version block in the YAML output
    #[serde(rename = "last-modified", skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub extension_type: Option<Vec<String>>,

    #[serde(
        deserialize_with = "deserialize_version",
        serialize_with = "serialize_version",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub version: Option<semver::Version>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(rename = "makefile-entries", skip_serializing_if = "Option::is_none")]
    pub makefile_entries: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

fn deserialize_version<'de, D>(deserializer: D) -> Result<Option<semver::Version>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version_str: String = String::deserialize(deserializer)?;
    let normalized = if version_str.eq_ignore_ascii_case("SNAPSHOT") {
        "0.0.0-SNAPSHOT".to_string()
    } else if version_str.matches('.').count() == 1 {
        format!("{}.0", version_str)
    } else {
        version_str
    };
    semver::Version::parse(&normalized).map(Some).map_err(serde::de::Error::custom)
}

fn serialize_version<S>(v: &Option<semver::Version>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match v {
        Some(ver)
            if ver.major == 0 && ver.minor == 0 && ver.patch == 0
                && ver.pre.as_str() == "SNAPSHOT" =>
        {
            s.serialize_str("SNAPSHOT")
        }
        Some(ver) => s.serialize_str(&ver.to_string()),
        None => unreachable!("skip_serializing_if guards None"),
    }
}
