use serde::Deserialize;


#[derive(Deserialize, Debug, Clone)]
pub struct Extensions {
    #[serde(rename = "schema-version")]
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

#[derive(Deserialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    pub name: String,
    pub author: Option<String>,
    pub desc: Option<String>,
    pub homepage: Option<String>,
    pub tags: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, PartialOrd, Ord, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    #[serde(rename = "type")]
    pub extension_type: Option<Vec<String>>,

    #[serde(deserialize_with = "deserialize_version", default)]
    pub version: Option<semver::Version>,

    pub url: Option<String>,

    #[serde(rename = "makefile-entries")]
    pub makefile_entries: Option<Vec<String>>,
    pub ext: Option<String>,
    pub branch: Option<String>,

    #[serde(rename = "last-modified")]
    pub last_modified: Option<String>,
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

