use std::fs;
use std::path::PathBuf;

use crate::args::InteractiveFictionSystem;

/// Detects the IF system from files in the current directory only (not recursive).
/// Returns the detected system and, for TADS3, the path to the first .t3m makefile found.
pub fn detect_system() -> (InteractiveFictionSystem, Option<PathBuf>) {
    detect_system_in(std::path::Path::new("."))
}

fn detect_system_in(dir: &std::path::Path) -> (InteractiveFictionSystem, Option<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return (InteractiveFictionSystem::Unknown, None),
    };

    let mut t3m: Option<PathBuf> = None;
    let mut has_inform = false;
    let mut has_inf    = false;
    let mut has_dg     = false;
    let mut has_hug    = false;
    let mut has_zil    = false;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        match path.extension().and_then(|s| s.to_str()) {
            Some("inform") if path.is_dir()  => { has_inform = true; }
            Some("t3m")    if path.is_file() => { t3m.get_or_insert(path); }
            Some("inf")    if path.is_file() => { has_inf = true; }
            Some("dg")     if path.is_file() => { has_dg  = true; }
            Some("hug")    if path.is_file() => { has_hug = true; }
            Some("zil")    if path.is_file() => { has_zil = true; }
            _ => {}
        }
    }

    if t3m.is_some()  { return (InteractiveFictionSystem::Tads3,   t3m); }
    if has_inform     { return (InteractiveFictionSystem::Inform,   None); }
    if has_inf        { return (InteractiveFictionSystem::Inform6,  None); }
    if has_dg         { return (InteractiveFictionSystem::Dialog,   None); }
    if has_hug        { return (InteractiveFictionSystem::Hugo,     None); }
    if has_zil        { return (InteractiveFictionSystem::Zil,      None); }

    (InteractiveFictionSystem::Unknown, None)
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn touch(dir: &TempDir, name: &str) {
        fs::write(dir.path().join(name), "").unwrap();
    }

    fn mkdir(dir: &TempDir, name: &str) {
        fs::create_dir(dir.path().join(name)).unwrap();
    }

    #[test]
    fn detect_t3m() {
        let d = tmp();
        touch(&d, "game.t3m");
        let (sys, path) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Tads3);
        assert!(path.is_some());
    }

    #[test]
    fn detect_inform_dir() {
        let d = tmp();
        mkdir(&d, "game.inform");
        let (sys, _) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Inform);
    }

    #[test]
    fn detect_inf_file() {
        let d = tmp();
        touch(&d, "game.inf");
        let (sys, _) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Inform6);
    }

    #[test]
    fn detect_dg_file() {
        let d = tmp();
        touch(&d, "game.dg");
        let (sys, _) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Dialog);
    }

    #[test]
    fn detect_hug_file() {
        let d = tmp();
        touch(&d, "game.hug");
        let (sys, _) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Hugo);
    }

    #[test]
    fn detect_zil_file() {
        let d = tmp();
        touch(&d, "game.zil");
        let (sys, _) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Zil);
    }

    #[test]
    fn detect_unknown_empty_dir() {
        let d = tmp();
        let (sys, path) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Unknown);
        assert!(path.is_none());
    }

    #[test]
    fn detect_unknown_unrecognised_files() {
        let d = tmp();
        touch(&d, "readme.txt");
        touch(&d, "game.z5");
        let (sys, _) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Unknown);
    }

    #[test]
    fn t3m_takes_priority_over_other_extensions() {
        let d = tmp();
        touch(&d, "game.t3m");
        touch(&d, "game.inf");
        touch(&d, "game.dg");
        let (sys, path) = detect_system_in(d.path());
        assert_eq!(sys, InteractiveFictionSystem::Tads3);
        assert!(path.is_some());
    }
}
