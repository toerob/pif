pub mod dir;
pub mod verbose;
pub mod systems;
pub mod versions;

pub use dir::*;
pub use verbose::*;
pub use systems::*;
pub use versions::*;

use dirs_next::config_dir;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::Deserialize;
use serde_yaml::{Mapping, Value};

#[derive(Deserialize, Default)]
pub struct PifConfig {
    #[serde(default)]
    pub install_dirs: HashMap<String, String>,
    pub verbose_level: Option<usize>,
    #[serde(default)]
    pub systems: Vec<String>,
    #[serde(default)]
    pub system_versions: HashMap<String, Vec<String>>,
}

pub enum VersionSpec {
    Prefix(String),
    // Future variants: AtLeast, LessThan, Caret, Tilde, …
}

impl VersionSpec {
    pub fn parse(s: &str) -> Self {
        VersionSpec::Prefix(s.to_string())
    }

    pub fn matches(&self, version: &str) -> bool {
        match self {
            // Split on '-' so "i10" matches "16-i10.1" (Inform's version-branch format).
            VersionSpec::Prefix(p) => version.split('-').any(|seg| seg.starts_with(p.as_str())),
        }
    }
}

pub fn version_matches_any(version: &str, specs: &[VersionSpec]) -> bool {
    specs.is_empty() || specs.iter().any(|s| s.matches(version))
}

lazy_static::lazy_static! {
    pub static ref VERBOSE_DEFAULT: usize = load_config().verbose_level.unwrap_or(2);
}

pub fn load_config() -> PifConfig {
    match get_main_config_file() {
        Ok(path) => fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_yaml::from_str(&s).ok())
            .unwrap_or_default(),
        Err(_) => PifConfig::default(),
    }
}

/// Expand a leading `~/` to the user's home directory.
pub fn expand_path(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs_next::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(s)
}

pub fn get_main_config_file() -> Result<PathBuf, io::Error> {
    let config_dir = config_dir()
        .expect("Could not determine config directory")
        .join("pif")
        .join("config");

    if !config_dir.exists() {
        println!("Missing configuration directory, creating it.");
        fs::create_dir_all(&config_dir)?;
    }

    let config_file = config_dir.join("config.yaml");

    if !config_file.exists() {
        println!("No config file exists. Writing a default one");
        let default_content =
r#"main_repository_url: https://github.com/toerob/if-extensions
main_repository_branch: main

# Override the default installation directory per system.
# If omitted, inform uses the platform Extensions folder; all others use ".".
# install_dirs:
#   tads3: ~/path/to/tads3/extensions
#   dialog: ~/path/to/dialog/extensions
#   inform: ~/path/to/inform/extensions
#   inform6: ~/path/to/inform6/extensions

# Restrict list/search/tags to specific systems when --system is not given
# and no IF project is detected in the current directory.
# systems: [tads3, inform6, inform]

# Pin which versions are considered for a system.
# Each entry is a matcher string:
#   bare string  → prefix match  (e.g. "i10" matches i10.1, i10.2, …)
#   (future)     → >=, <, ^, ~ constraint syntax
# system_versions:
#   inform: [i10, i11.0]
#   tads3: [3.1, 3.2]
"#;
        fs::write(&config_file, default_content)?;
        println!("Default settings written to {:?}", config_file);
    }

    Ok(config_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── VersionSpec::matches ─────────────────────────────────────────────────

    #[test]
    fn prefix_matches_exact() {
        assert!(VersionSpec::parse("3.1").matches("3.1"));
    }

    #[test]
    fn prefix_matches_longer_plain_version() {
        assert!(VersionSpec::parse("3.1").matches("3.1.0"));
        assert!(VersionSpec::parse("3.1").matches("3.1.2"));
    }

    #[test]
    fn prefix_does_not_match_different_minor() {
        assert!(!VersionSpec::parse("3.1").matches("3.2.0"));
        assert!(!VersionSpec::parse("3.1").matches("2.1.0"));
    }

    #[test]
    fn prefix_matches_inform_branch_segment() {
        // Inform release files are named v16-i10.1.yaml → version "16-i10.1"
        assert!(VersionSpec::parse("i10").matches("16-i10.1"));
        assert!(VersionSpec::parse("i10").matches("16-i10.2"));
        assert!(VersionSpec::parse("i11").matches("16-i11.0"));
    }

    #[test]
    fn prefix_does_not_match_wrong_inform_branch() {
        assert!(!VersionSpec::parse("i10").matches("16-i11.0"));
        assert!(!VersionSpec::parse("i11").matches("16-i10.1"));
    }

    #[test]
    fn prefix_matches_inform_branch_without_version_prefix() {
        // Branch-only version string (no leading "16-")
        assert!(VersionSpec::parse("i10").matches("i10.1"));
        assert!(!VersionSpec::parse("i10").matches("i11.0"));
    }

    #[test]
    fn prefix_matches_specific_branch_point() {
        assert!(VersionSpec::parse("i10.1").matches("16-i10.1"));
        assert!(!VersionSpec::parse("i10.1").matches("16-i10.2"));
    }

    // ── version_matches_any ──────────────────────────────────────────────────

    #[test]
    fn empty_specs_always_matches() {
        assert!(version_matches_any("anything", &[]));
        assert!(version_matches_any("16-i10.1", &[]));
    }

    #[test]
    fn single_spec_match_and_no_match() {
        let specs = [VersionSpec::parse("i10")];
        assert!(version_matches_any("16-i10.1", &specs));
        assert!(!version_matches_any("16-i11.0", &specs));
    }

    #[test]
    fn multiple_specs_are_or_combined() {
        let specs = [VersionSpec::parse("i10"), VersionSpec::parse("i11")];
        assert!(version_matches_any("16-i10.1", &specs));
        assert!(version_matches_any("16-i11.0", &specs));
        assert!(!version_matches_any("16-i12.0", &specs));
    }

    #[test]
    fn all_specs_failing_returns_false() {
        let specs = [VersionSpec::parse("i10"), VersionSpec::parse("i11")];
        assert!(!version_matches_any("16-i9.0", &specs));
    }
}

pub(super) fn load_yaml() -> Result<(PathBuf, serde_yaml::Value), Box<dyn std::error::Error>> {
    let path = get_main_config_file()?;
    let content = fs::read_to_string(&path)?;
    let root = serde_yaml::from_str(&content)
        .unwrap_or(Value::Mapping(Mapping::new()));
    Ok((path, root))
}

pub(super) fn save_yaml(path: &PathBuf, root: &serde_yaml::Value) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, serde_yaml::to_string(root)?)?;
    Ok(())
}
