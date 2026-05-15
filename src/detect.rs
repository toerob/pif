use std::fs;
use std::path::PathBuf;

use crate::args::InteractiveFictionSystem;

/// Detects the IF system from files in the current directory only (not recursive).
/// Returns the detected system and, for TADS3, the path to the first .t3m makefile found.
pub fn detect_system() -> (InteractiveFictionSystem, Option<PathBuf>) {
    let entries = match fs::read_dir(".") {
        Ok(e) => e,
        Err(_) => return (InteractiveFictionSystem::Unknown, None),
    };

    let mut t3m: Option<PathBuf> = None;
    let mut has_inform = false;
    let mut has_inf    = false;
    let mut has_dg     = false;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        match path.extension().and_then(|s| s.to_str()) {
            Some("inform") if path.is_dir() => { has_inform = true; }
            Some("t3m") if path.is_file()   => { t3m.get_or_insert(path); }
            Some("inf") if path.is_file()   => { has_inf = true; }
            Some("dg")  if path.is_file()   => { has_dg  = true; }
            _ => {}
        }
    }

    if t3m.is_some()  { return (InteractiveFictionSystem::Tads3,   t3m); }
    if has_inform     { return (InteractiveFictionSystem::Inform,   None); }
    if has_inf        { return (InteractiveFictionSystem::Inform6,  None); }
    if has_dg         { return (InteractiveFictionSystem::Dialog,   None); }

    (InteractiveFictionSystem::Unknown, None)
}


#[cfg(test)]
mod tests {
    

    #[test]
    fn get_extension_path_works() {
    }
}
