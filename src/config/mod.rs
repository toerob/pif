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
    // Test hook: lets tests redirect the config file to a temp path.
    if let Ok(p) = std::env::var("PIF_TEST_CONFIG") {
        return Ok(PathBuf::from(p));
    }

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
    // Why tests live here rather than in tests/
    // ─────────────────────────────────────────
    // `pif` is a binary crate (main.rs, no lib.rs). Rust's tests/ directory
    // only links against library crates, so `use pif::config::...` from there
    // simply doesn't compile. Inline tests are compiled as part of the binary
    // and can see everything, including pub(super) helpers like load_yaml.

    use super::*;
    use std::sync::Mutex;

    // Why the lock
    // ────────────
    // Rust runs tests in parallel across threads. PIF_TEST_CONFIG is a
    // process-level env var shared by all threads. Without the lock, two tests
    // could race: one sets the var to path A, another to path B, and both end
    // up operating on the wrong file. Every test that touches TestConfig must
    // hold this lock for its entire duration.
    static LOCK: Mutex<()> = Mutex::new(());

    // Why _g and not just LOCK.lock()
    // ────────────────────────────────
    // A temporary with no binding is dropped at the end of its statement.
    // `let _g = LOCK.lock()` keeps the guard alive until end of scope.
    // `let _ = LOCK.lock()` would release it immediately — never use that form.

    struct TestConfig {
        // TempDir deletes the directory when dropped. Named _dir (not _) so
        // Rust keeps it alive for the struct's lifetime rather than dropping
        // it immediately at the end of new().
        _dir: tempfile::TempDir,
        path: PathBuf,
    }

    impl TestConfig {
        // Writes `yaml` into a temp file, then sets PIF_TEST_CONFIG to that
        // path. Every config function in this process now reads/writes that
        // file instead of the real ~/.config/pif/config/config.yaml.
        fn new(yaml: &str) -> Self {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("config.yaml");
            fs::write(&path, yaml).unwrap();
            std::env::set_var("PIF_TEST_CONFIG", &path);
            TestConfig { _dir: dir, path }
        }

        // Re-reads the file from disk so tests can inspect what was actually
        // written by the function under test.
        fn read_yaml(&self) -> serde_yaml::Value {
            let s = fs::read_to_string(&self.path).unwrap();
            serde_yaml::from_str(&s).unwrap()
        }

        // Convenience wrapper for reading a top-level YAML sequence as strings.
        fn seq(&self, key: &str) -> Vec<String> {
            self.read_yaml()[key]
                .as_sequence()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        }
    }

    impl Drop for TestConfig {
        // Clears PIF_TEST_CONFIG whether the test passed, panicked, or returned
        // early. Without this, a panic would leave the var pointing at a deleted
        // temp path and corrupt the next test to acquire the lock.
        fn drop(&mut self) {
            std::env::remove_var("PIF_TEST_CONFIG");
        }
    }

    fn system_versions(yaml: &serde_yaml::Value, system: &str) -> Vec<String> {
        yaml["system_versions"][system]
            .as_sequence()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    }

    fn install_dir(yaml: &serde_yaml::Value, system: &str) -> Option<String> {
        yaml["install_dirs"][system].as_str().map(String::from)
    }

    const EMPTY: &str = "{}\n";

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

    // ── systems.rs ───────────────────────────────────────────────────────────

    #[test]
    fn systems_set_replaces_existing() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new("systems:\n  - tads3\n");
        set_systems(&["inform".into(), "dialog".into()]).unwrap();
        assert_eq!(cfg.seq("systems"), ["inform", "dialog"]);
    }

    #[test]
    fn systems_add_appends_without_duplicates() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new("systems:\n  - tads3\n");
        add_systems(&["tads3".into(), "inform".into()]).unwrap();
        assert_eq!(cfg.seq("systems"), ["tads3", "inform"]);
    }

    #[test]
    fn systems_add_to_empty_config() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new(EMPTY);
        add_systems(&["dialog".into()]).unwrap();
        assert_eq!(cfg.seq("systems"), ["dialog"]);
    }

    #[test]
    fn systems_remove_listed_entries() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new("systems:\n  - tads3\n  - inform\n  - dialog\n");
        let (_, removed) = remove_systems(&["tads3".into(), "dialog".into()]).unwrap();
        assert_eq!(removed, ["tads3", "dialog"]);
        assert_eq!(cfg.seq("systems"), ["inform"]);
    }

    #[test]
    fn systems_remove_absent_entry_returns_empty() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new("systems:\n  - tads3\n");
        let (_, removed) = remove_systems(&["inform".into()]).unwrap();
        assert!(removed.is_empty());
        assert_eq!(cfg.seq("systems"), ["tads3"]);
    }

    #[test]
    fn systems_reset_removes_key() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new("systems:\n  - tads3\n");
        let (_, was_present) = reset_systems().unwrap();
        assert!(was_present);
        assert!(cfg.read_yaml().get("systems").is_none());
    }

    #[test]
    fn systems_reset_on_empty_config_returns_false() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new(EMPTY);
        let (_, was_present) = reset_systems().unwrap();
        assert!(!was_present);
        let _ = cfg;
    }

    // ── versions.rs ──────────────────────────────────────────────────────────

    // Why version strings must be quoted in hand-written YAML
    // ────────────────────────────────────────────────────────
    // Bare "3.1" in YAML is parsed as a float, not a string. The config
    // functions store versions as Value::String, so a lookup for "3.1"
    // against a float 3.1 finds nothing and silently does the wrong thing.
    // Always quote numeric-looking version strings in test fixtures: "3.1".

    #[test]
    fn versions_set_replaces_for_system() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new("system_versions:\n  tads3:\n    - \"3.1\"\n");
        set_system_versions("tads3", &["3.2".into(), "3.3".into()]).unwrap();
        assert_eq!(system_versions(&cfg.read_yaml(), "tads3"), ["3.2", "3.3"]);
    }

    #[test]
    fn versions_add_appends_without_duplicates() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new("system_versions:\n  tads3:\n    - \"3.1\"\n");
        add_system_version_specs("tads3", &["3.1".into(), "3.2".into()]).unwrap();
        assert_eq!(system_versions(&cfg.read_yaml(), "tads3"), ["3.1", "3.2"]);
    }

    #[test]
    fn versions_add_creates_system_entry() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new(EMPTY);
        add_system_version_specs("inform", &["i10".into()]).unwrap();
        assert_eq!(system_versions(&cfg.read_yaml(), "inform"), ["i10"]);
    }

    #[test]
    fn versions_remove_listed_entries() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new(
            "system_versions:\n  tads3:\n    - \"3.1\"\n    - \"3.2\"\n    - \"3.3\"\n",
        );
        let (_, removed) =
            remove_system_version_specs("tads3", &["3.1".into(), "3.3".into()]).unwrap();
        assert_eq!(removed, ["3.1", "3.3"]);
        assert_eq!(system_versions(&cfg.read_yaml(), "tads3"), ["3.2"]);
    }

    #[test]
    fn versions_reset_one_system() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new(
            "system_versions:\n  tads3:\n    - \"3.1\"\n  inform:\n    - i10\n",
        );
        let (_, removed) = reset_system_versions(Some("tads3")).unwrap();
        assert!(removed);
        let yaml = cfg.read_yaml();
        assert!(yaml["system_versions"].get("tads3").is_none());
        assert_eq!(system_versions(&yaml, "inform"), ["i10"]);
    }

    #[test]
    fn versions_reset_all() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new(
            "system_versions:\n  tads3:\n    - \"3.1\"\n  inform:\n    - i10\n",
        );
        let (_, removed) = reset_system_versions(None).unwrap();
        assert!(removed);
        assert!(cfg.read_yaml().get("system_versions").is_none());
    }

    // ── dir.rs ───────────────────────────────────────────────────────────────

    #[test]
    fn dir_set_writes_directory() {
        let _g = LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let install_path = tmp.path().join("tads3-ext");
        fs::create_dir_all(&install_path).unwrap();
        let cfg = TestConfig::new(EMPTY);
        set_install_dir("tads3", install_path.to_str().unwrap()).unwrap();
        assert_eq!(
            install_dir(&cfg.read_yaml(), "tads3").as_deref(),
            Some(install_path.to_str().unwrap()),
        );
    }

    #[test]
    fn dir_reset_removes_entry() {
        let _g = LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let install_path = tmp.path().join("ext");
        fs::create_dir_all(&install_path).unwrap();
        let cfg = TestConfig::new(EMPTY);
        set_install_dir("dialog", install_path.to_str().unwrap()).unwrap();
        let (_, removed) = reset_install_dir("dialog").unwrap();
        assert!(removed);
        assert!(install_dir(&cfg.read_yaml(), "dialog").is_none());
    }

    #[test]
    fn dir_reset_absent_entry_returns_false() {
        let _g = LOCK.lock().unwrap();
        let cfg = TestConfig::new(EMPTY);
        let (_, removed) = reset_install_dir("tads3").unwrap();
        assert!(!removed);
        let _ = cfg;
    }

    #[test]
    fn dir_reset_all_clears_everything() {
        let _g = LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let p1 = tmp.path().join("a");
        let p2 = tmp.path().join("b");
        fs::create_dir_all(&p1).unwrap();
        fs::create_dir_all(&p2).unwrap();
        let cfg = TestConfig::new(EMPTY);
        set_install_dir("tads3", p1.to_str().unwrap()).unwrap();
        set_install_dir("dialog", p2.to_str().unwrap()).unwrap();
        let (_, count) = reset_all_install_dirs().unwrap();
        assert_eq!(count, 2);
        assert!(cfg.read_yaml().get("install_dirs").map_or(true, |v| {
            v.as_mapping().map_or(true, |m| m.is_empty())
        }));
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
