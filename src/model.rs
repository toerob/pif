use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// ── Registry location (pif-index bootstrap) ───────────────────────────────

#[derive(Deserialize, Debug)]
pub struct RegistryLocation {
    pub version: u32,
    pub url: String,
    pub root: String,
    pub branch: Option<String>,
    pub message: Option<String>,
}

// ── YAML-mapped structs ────────────────────────────────────────────────────

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Package {
    #[serde(rename = "schema-version")]
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub author: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Release {
    #[serde(rename = "schema-version")]
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<HashMap<String, CompatibilityConstraint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<Dependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<Build>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CompatibilityConstraint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Source {
    pub url: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Build {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<Vec<BuildEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<Vec<BuildEntry>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BuildEntry {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Dependency {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<String>,
}

// ── Derived structs ────────────────────────────────────────────────────────

/// A release loaded from disk, with the version string parsed from its filename.
/// For Inform releases the filename is `v{version}-i{branch}.yaml`; the branch
/// suffix is stripped so `version` holds only the version part (e.g. "16").
#[derive(Debug, Clone)]
pub struct LoadedRelease {
    pub version: String,
    pub release: Release,
}

/// A fully loaded package together with all its releases.
#[derive(Debug, Clone)]
pub struct PackageEntry {
    pub system: String,
    #[allow(dead_code)] 
    pub namespace: String,
    pub package: Package,
    pub releases: Vec<LoadedRelease>,
}

// ── Schema validation ──────────────────────────────────────────────────────

/// Validates `release` against the `release.schema.yaml` bundled with the registry.
/// Returns a list of human-readable error strings.
/// Returns an empty vec if the schema file is absent (e.g. before first `pif update`).
pub fn validate_release_schema(release: &Release, repo_dir: &Path) -> Vec<String> {
    let schema_path = repo_dir.join("schemas").join("release.schema.yaml");
    validate_against_schema(release, &schema_path)
}

fn validate_against_schema<T: Serialize>(value: &T, schema_path: &Path) -> Vec<String> {
    let schema_str = match fs::read_to_string(schema_path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let schema_yaml: serde_yaml::Value = match serde_yaml::from_str(&schema_str) {
        Ok(v) => v,
        Err(e) => return vec![format!("could not parse schema file: {}", e)],
    };

    let schema_json: JsonValue = match serde_json::to_value(&schema_yaml) {
        Ok(v) => v,
        Err(e) => return vec![format!("could not convert schema to JSON: {}", e)],
    };

    let instance: JsonValue = match serde_json::to_value(value) {
        Ok(v) => v,
        Err(e) => return vec![format!("could not serialize release for validation: {}", e)],
    };

    let validator = match jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema_json)
    {
        Ok(v) => v,
        Err(e) => return vec![format!("could not compile schema: {}", e)],
    };

    validator.iter_errors(&instance).map(|e| {
        let path = e.instance_path.to_string();
        if path.is_empty() {
            e.to_string()
        } else {
            format!("{}: {}", path, e)
        }
    }).collect()
}

// ── Registry loader ────────────────────────────────────────────────────────

/// Walk `registry_root` and return every package entry found.
/// Pass `system_filter` to restrict to a single system (e.g. `"tads3"` or `"inform"`).
pub fn load_registry(
    registry_root: &Path,
    system_filter: Option<&str>,
) -> Result<Vec<PackageEntry>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();

    for system_entry in sorted_dirs(registry_root)? {
        let system = dir_name(&system_entry);
        if let Some(filter) = system_filter {
            if system != filter {
                continue;
            }
        }

        for ns_entry in sorted_dirs(&system_entry)? {
            let namespace = dir_name(&ns_entry);

            for pkg_entry in sorted_dirs(&ns_entry)? {
                let package_path = pkg_entry.join("package.yaml");
                if !package_path.exists() {
                    continue;
                }

                let package: Package = match fs::read_to_string(&package_path)
                    .map_err(|e| e.to_string())
                    .and_then(|s| serde_yaml::from_str(&s).map_err(|e| e.to_string()))
                {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Warning: skipping {:?}: {}", package_path, e);
                        continue;
                    }
                };

                let releases = match load_releases(&pkg_entry.join("releases")) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Warning: skipping releases for {:?}: {}", pkg_entry, e);
                        continue;
                    }
                };

                entries.push(PackageEntry {
                    system: system.clone(),
                    namespace: namespace.clone(),
                    package,
                    releases,
                });
            }
        }
    }

    Ok(entries)
}

fn load_releases(releases_dir: &Path) -> Result<Vec<LoadedRelease>, Box<dyn std::error::Error>> {
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut releases = Vec::new();

    let mut paths: Vec<_> = fs::read_dir(releases_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    paths.sort();

    for path in paths {
        let version = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .trim_start_matches('v')
            .to_string();

        let release: Release = serde_yaml::from_str(&fs::read_to_string(&path)?)?;
        releases.push(LoadedRelease { version, release });
    }

    Ok(releases)
}

fn sorted_dirs(parent: &Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
    let mut dirs: Vec<_> = fs::read_dir(parent)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    Ok(dirs)
}

fn dir_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}
