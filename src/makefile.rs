use std::fs;
use std::path::Path;

use crate::common::yes_or_no;
use ansi_term::Colour::*;

enum EntryKind {
    Define { key: String, directive: String },
    Lib(String),
    Source(String),
    Unknown(String),
}

fn classify(entry: &str) -> EntryKind {
    if entry.starts_with("-lib ") {
        return EntryKind::Lib(entry.to_string());
    }
    if entry.starts_with("-source ") {
        return EntryKind::Source(entry.to_string());
    }
    if entry.starts_with("-D ") {
        let key = entry[3..].split('=').next().unwrap_or("").trim().to_string();
        return EntryKind::Define { key, directive: entry.to_string() };
    }
    EntryKind::Unknown(entry.to_string())
}

// Returns the index of the last line starting with "-D ", or None.
fn last_define_line(lines: &[String]) -> Option<usize> {
    lines.iter().enumerate()
        .filter(|(_, l)| l.starts_with("-D "))
        .map(|(i, _)| i)
        .last()
}

// Returns the index of the first -lib or -source line, or None.
fn first_lib_or_source_line(lines: &[String]) -> Option<usize> {
    lines.iter().enumerate()
        .find(|(_, l)| l.starts_with("-lib ") || l.starts_with("-source "))
        .map(|(i, _)| i)
}

fn last_lib_line(lines: &[String]) -> usize {
    lines.iter().enumerate()
        .filter(|(_, l)| l.starts_with("-lib "))
        .map(|(i, _)| i)
        .last()
        .unwrap_or(0)
}

fn last_source_line(lines: &[String]) -> usize {
    lines.iter().enumerate()
        .filter(|(_, l)| l.starts_with("-source "))
        .map(|(i, _)| i)
        .last()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    // classify
    #[test]
    fn classify_lib() {
        assert!(matches!(classify("-lib foo.a"), EntryKind::Lib(_)));
    }

    #[test]
    fn classify_source() {
        assert!(matches!(classify("-source bar.c"), EntryKind::Source(_)));
    }

    #[test]
    fn classify_define() {
        let EntryKind::Define { key, directive } = classify("-D FOO=1") else { panic!() };
        assert_eq!(key, "FOO");
        assert_eq!(directive, "-D FOO=1");
    }

    #[test]
    fn classify_unknown() {
        assert!(matches!(classify("-other thing"), EntryKind::Unknown(_)));
    }

    // last_define_line
    #[test]
    fn last_define_line_empty() {
        assert_eq!(last_define_line(&[]), None);
    }

    #[test]
    fn last_define_line_no_match() {
        assert_eq!(last_define_line(&s(&["-lib foo", "-source bar"])), None);
    }

    #[test]
    fn last_define_line_single() {
        assert_eq!(last_define_line(&s(&["-D A=1"])), Some(0));
    }

    #[test]
    fn last_define_line_multiple() {
        assert_eq!(last_define_line(&s(&["-D A=1", "-lib x", "-D B=2"])), Some(2));
    }

    // first_lib_or_source_line
    #[test]
    fn first_lib_or_source_first_lib() {
        assert_eq!(first_lib_or_source_line(&s(&["-lib a", "-source b"])), Some(0));
    }

    #[test]
    fn first_lib_or_source_first_source() {
        assert_eq!(first_lib_or_source_line(&s(&["-D X=1", "-source b"])), Some(1));
    }

    #[test]
    fn first_lib_or_source_mixed_ordering() {
        assert_eq!(first_lib_or_source_line(&s(&["-D A=1", "-D B=2", "-source first", "-lib later"])), Some(2));
    }

    #[test]
    fn first_lib_or_source_none() {
        assert_eq!(first_lib_or_source_line(&s(&["-D X=1"])), None);
    }

    // last_lib_line
    #[test]
    fn last_lib_line_absent() {
        assert_eq!(last_lib_line(&s(&["-source a"])), 0);
    }

    #[test]
    fn last_lib_line_single() {
        assert_eq!(last_lib_line(&s(&["-lib a"])), 0);
    }

    #[test]
    fn last_lib_line_multiple() {
        assert_eq!(last_lib_line(&s(&["-lib a", "-D X=1", "-lib b"])), 2);
    }

    // last_source_line
    #[test]
    fn last_source_line_absent() {
        assert_eq!(last_source_line(&s(&["-lib a"])), 0);
    }

    #[test]
    fn last_source_line_single() {
        assert_eq!(last_source_line(&s(&["-source a"])), 0);
    }

    #[test]
    fn last_source_line_multiple() {
        assert_eq!(last_source_line(&s(&["-source a", "-lib x", "-source b"])), 2);
    }
}

pub fn add_make_file_entry(_name: String, makefile: &Path, build_entries: Vec<String>) {
    let contents = fs::read_to_string(makefile).expect("Could not read the makefile");

    let mut lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
    let mut diff_lines = lines.clone();
    let mut any_change = false;

    // Warn about entries that don't follow the required format.
    for entry in &build_entries {
        if let EntryKind::Unknown(e) = classify(entry) {
            eprintln!("Skipping unrecognised build-entry '{}': must start with -lib, -source, or -D", e);
        }
    }

    // Pass 1: -D defines — replace in-place or insert before the lib block.
    for entry in &build_entries {
        let EntryKind::Define { key, directive } = classify(entry) else { continue };

        // Look for an existing "-D KEY=..." or "-D KEY" line to replace.
        let existing_idx = lines.iter().position(|l| {
            l.starts_with("-D ") && {
                let rest = l[3..].trim();
                rest == key || rest.starts_with(&format!("{}=", key)) || rest.starts_with(&format!("{} ", key))
            }
        });

        if let Some(idx) = existing_idx {
            if lines[idx] == directive {
                continue; // already correct
            }
            diff_lines[idx] = format!("{} {} {}", Red.paint(&lines[idx]), Yellow.paint("=>"), Green.paint(&directive));
            lines[idx] = directive;
            any_change = true;
        } else {
            // Insert after the last -D line, or just before the first -lib/-source, or at position 0.
            let insert_at = last_define_line(&lines)
                .map(|i| i + 1)
                .or_else(|| first_lib_or_source_line(&lines))
                .unwrap_or(0);

            diff_lines.insert(insert_at, Green.paint(&directive).to_string());
            lines.insert(insert_at, directive);
            any_change = true;
        }
    }

    // Pass 2: -lib entries — insert after the last -lib line.
    for entry in &build_entries {
        let EntryKind::Lib(directive) = classify(entry) else { continue };
        if lines.iter().any(|l| l == &directive) {
            continue;
        }
        let insert_at = last_lib_line(&lines) + 1;
        diff_lines.insert(insert_at, Green.paint(&directive).to_string());
        lines.insert(insert_at, directive);
        any_change = true;
    }

    // Pass 3: -source entries — insert after the last -source line.
    for entry in &build_entries {
        let EntryKind::Source(directive) = classify(entry) else { continue };
        if lines.iter().any(|l| l == &directive) {
            continue;
        }
        let insert_at = last_source_line(&lines) + 1;
        diff_lines.insert(insert_at, Green.paint(&directive).to_string());
        lines.insert(insert_at, directive);
        any_change = true;
    }

    if !any_change {
        return;
    }

    println!("{}", Yellow.paint("Makefile suggested contents:"));
    print!("{}\n\n", diff_lines.join("\n"));

    if yes_or_no("Apply above changes?", true) {
        if makefile.exists() {
            fs::write(makefile, lines.join("\n"))
                .expect("Makefile could not be found. Skipping. ");
            println!("Changes applied");
        }
    } else {
        println!("No changes applied");
    }
    println!();
}
