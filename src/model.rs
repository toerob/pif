use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

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
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Release {
    #[serde(rename = "schema-version")]
    pub schema_version: u32,
    pub maintainer: Option<String>,
    pub channel: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    pub compatibility: Option<HashMap<String, CompatibilityConstraint>>,
    pub dependencies: Option<Vec<Dependency>>,
    pub source: Option<Source>,
    pub build: Option<Build>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CompatibilityConstraint {
    pub constraint: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Source {
    pub url: String,
    pub format: String,
    pub branch: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Build {
    pub exports: Option<Vec<BuildEntry>>,
    pub private: Option<Vec<BuildEntry>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BuildEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub path: Option<String>,
    pub value: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Dependency {
    pub id: String,
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
    pub namespace: String,
    pub package: Package,
    pub releases: Vec<LoadedRelease>,
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

                let package: Package =
                    serde_yaml::from_str(&fs::read_to_string(&package_path)?)?;

                let releases = load_releases(&pkg_entry.join("releases"))?;

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
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .trim_start_matches('v');

        // Strip inform branch suffix: "16-i10.1" → "16"
        let version = match stem.find("-i") {
            Some(pos) => &stem[..pos],
            None => stem,
        }
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
