use ansi_term::Colour::*;
use std::{fs::{self, File}, io::{Cursor, Write}, path::Path, process::exit};
use sublime_fuzzy::FuzzySearch;

use std::path::PathBuf;

use crate::{
    args::{Color, GlobalOptions, InstallOptions, InteractiveFictionSystem},
    color::{print_success_msg, print_warning_msg},
    detect::detect_system,
    gitops::{get_or_create_repo_dir, clone_or_pull_repo},
    list::system_to_dir,
    makefile::add_make_file_entry,
    model::{load_registry, BuildEntry, LoadedRelease},
    update::{get_registry_root, update_extensions},
    db::{get_or_create_table, record_installation},
};

pub fn install_extensions(
    names: &Vec<String>,
    install_options: &InstallOptions,
    global_options: &GlobalOptions,
    update_needed: bool,
) {
    if update_needed {
        update_extensions(global_options);
    }

    let use_colours = Color::Never != global_options.color;

    if names.is_empty() {
        println!("{}", Red.paint(
            "No packages specified. Example:\n  pif install smarter-parser\n  pif install smarter-parser:16"
        ));
        return;
    }

    let (system_type, makefile): (InteractiveFictionSystem, Option<PathBuf>) =
        if global_options.system == InteractiveFictionSystem::Auto {
            detect_system()
        } else {
            (global_options.system.clone(), None)
        };

    //println!("{}", Yellow.paint(format!("System: {:?}", system_type)));
    if let Some(ref mf) = makefile {
        println!("{}", Yellow.paint(format!("Makefile: {}", mf.display())));
    }

    let system_filter = system_to_dir(&system_type);

    let registry_root = get_registry_root();

    let entries = match load_registry(&registry_root, system_filter) {
        Ok(e) => e,
        Err(e) => { eprintln!("Could not load registry: {}", e); return; }
    };

    // Parse "name:version" pairs
    let requests: Vec<(String, String)> = names.iter().map(|x| {
        let lower = x.to_lowercase();
        let mut parts = lower.splitn(2, ':');
        let name = parts.next().unwrap_or("").to_string();
        let ver  = parts.next().unwrap_or("latest").to_string();
        (name, ver)
    }).collect();

    let explicit_library_path: Option<PathBuf> = install_options.installation_directory
        .as_deref()
        .map(PathBuf::from);

    for (req_name, req_version) in &requests {
        let found = entries.iter().find(|e| {
            e.package.id.to_lowercase() == *req_name
                || e.package.name.to_lowercase() == *req_name
        }).or_else(|| {
            // fuzzy fallback
            entries.iter().find(|e| {
                FuzzySearch::new(req_name, &e.package.name.to_lowercase())
                    .case_insensitive().best_match().is_some()
            })
        });

        let entry = match found {
            Some(e) => e,
            None => {
                print_warning_msg(use_colours, format!("No extension found for '{}'\n", req_name));
                continue;
            }
        };

        let release = resolve_version(&entry.releases, req_version);
        let loaded = match release {
            Some(r) => r,
            None => {
                print_warning_msg(use_colours, format!(
                    "No matching version '{}' for '{}'\n", req_version, entry.package.name
                ));
                exit(0);
            }
        };

        let source = match &loaded.release.source {
            Some(s) => s,
            None => {
                print_warning_msg(use_colours, format!(
                    "'{}' has no source URL\n", entry.package.name
                ));
                continue;
            }
        };

        if let Some(deps) = &loaded.release.dependencies {
            if !deps.is_empty() {
                let list = deps.iter().map(|d| match &d.constraint {
                    Some(c) => format!("{} ({})", d.id, c),
                    None    => d.id.clone(),
                }).collect::<Vec<_>>().join(", ");
                println!("{}", Yellow.paint(format!("Note: '{}' depends on: {}", entry.package.name, list)));
                println!("{}", Yellow.paint("      Dependencies are not installed automatically yet."));
            }
        }

        let is_inform = entry.system == "inform";
        let library_path = explicit_library_path.clone()
            .or_else(|| if is_inform { inform_extensions_dir() } else { None })
            .unwrap_or_else(|| PathBuf::from("."));


        let install_path = if is_inform {
            library_path.join(&entry.package.author)
        } else {
            library_path.join(&entry.package.name)
        };
        let install_path_str = install_path.to_str().unwrap().to_owned();

        if !install_path.exists() {
            if let Err(e) = fs::create_dir_all(&install_path) {
                print_warning_msg(use_colours, format!("Could not create {}: {}\n", install_path_str, e));
                continue;
            }
        }

        let ok = match source.format.as_str() {
            "zip" => install_zip(&source.url, &install_path, !is_inform, use_colours),
            "git" => install_git(&source.url, source.branch.as_deref(), &install_path, use_colours),
            fmt   => install_raw_file(
                &source.url, &install_path, fmt,
                if is_inform { Some(&entry.package.name) } else { None },
                use_colours,
            ),
        };

        if !ok { continue; }

        record_entry(&entry.package.name, &install_path_str, &loaded.version, use_colours);
        print_success_msg(use_colours, format!(
            " ==> {} v{} [{}] installed into {}\n", entry.package.name, loaded.version, 
            entry.system.to_string(),
            install_path_str
        ));

        if system_type == InteractiveFictionSystem::Tads3 {
            if let Some(ref mf) = makefile {
                if let Some(build) = &loaded.release.build {
                    let flags = build_entries_to_flags(build.exports.as_deref().unwrap_or(&[]));
                    if !flags.is_empty() {
                        add_make_file_entry(entry.package.name.clone(), mf.as_path(), flags);
                    }
                }
            }
        }
    }
}

fn resolve_version<'a>(releases: &'a [LoadedRelease], version: &str) -> Option<&'a LoadedRelease> {
    if version == "latest" || version.is_empty() {
        return releases.iter().max_by(|a, b| version_ord(&a.version).cmp(&version_ord(&b.version)));
    }
    releases.iter().find(|r| r.version == version)
}

fn install_zip(url: &str, dest: &Path, strip_toplevel: bool, use_colours: bool) -> bool {
    let response = match reqwest::blocking::get(url) {
        Ok(r) => r,
        Err(e) => {
            print_warning_msg(use_colours, format!("Download failed: {}\n", e));
            return false;
        }
    };
    let bytes = match response.bytes() {
        Ok(b) => b,
        Err(e) => { print_warning_msg(use_colours, format!("Invalid response: {}\n", e)); return false; }
    };
    zip_extract::extract(Cursor::new(bytes), dest, strip_toplevel)
        .map_err(|e| print_warning_msg(use_colours, format!("Extraction failed: {}\n", e)))
        .is_ok()
}

fn install_git(url: &str, branch: Option<&str>, dest: &Path, use_colours: bool) -> bool {
    let branch = branch.unwrap_or("master");
    let dest_str = dest.to_str().unwrap();
    match get_or_create_repo_dir(dest_str) {
        Ok(repo_path) => {
            clone_or_pull_repo(url, branch, &repo_path)
                .map_err(|e| print_warning_msg(use_colours, format!("Git error: {}\n", e)))
                .is_ok()
        }
        Err(e) => { eprintln!("Could not create directory: {}", e); false }
    }
}

/// Download a single file. GitHub blob URLs are converted to raw.githubusercontent.com.
/// If `name_override` is given it is used as the filename stem instead of the URL-derived name.
fn install_raw_file(url: &str, dest: &Path, ext: &str, name_override: Option<&str>, use_colours: bool) -> bool {
    let raw_url = to_raw_github_url(url);
    let stem = name_override.unwrap_or_else(|| raw_url.rsplit('/').next().unwrap_or("extension"));
    let file_path = dest.join(stem).with_extension(ext);

    let response = match reqwest::blocking::get(raw_url.as_str()) {
        Ok(r) => r,
        Err(e) => { print_warning_msg(use_colours, format!("Download failed: {}\n", e)); return false; }
    };
    let bytes = match response.bytes() {
        Ok(b) => b,
        Err(e) => { print_warning_msg(use_colours, format!("Invalid response: {}\n", e)); return false; }
    };
    let mut file = match File::create(&file_path) {
        Ok(f) => f,
        Err(e) => { print_warning_msg(use_colours, format!("Could not create file: {}\n", e)); return false; }
    };
    file.write_all(&bytes)
        .map_err(|e| print_warning_msg(use_colours, format!("Write failed: {}\n", e)))
        .is_ok()
}

/// Convert a GitHub blob URL to a raw download URL.
/// https://github.com/owner/repo/blob/branch/path → https://raw.githubusercontent.com/owner/repo/branch/path
fn to_raw_github_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        if let Some(blob_pos) = rest.find("/blob/") {
            let (repo, path) = rest.split_at(blob_pos);
            let path = &path["/blob/".len()..];
            return format!("https://raw.githubusercontent.com/{}/{}", repo, path);
        }
    }
    url.to_string()
}

fn build_entries_to_flags(entries: &[BuildEntry]) -> Vec<String> {
    entries.iter().filter_map(|e| {
        match e.kind.as_str() {
            "lib"    => e.path.as_ref().map(|p| format!("-lib {}", p)),
            "source" => e.path.as_ref().map(|p| format!("-source {}", p)),
            "define" => e.value.as_ref().map(|v| format!("-D {}", v)),
            _        => None,
        }
    }).collect()
}

fn record_entry(name: &str, path: &str, version: &str, use_colours: bool) {
    if let Ok(conn) = get_or_create_table() {
        record_installation(&conn, name, path, version);
    } else {
        print_warning_msg(use_colours, "Could not access install registry db\n".into());
    }
}

fn inform_extensions_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return dirs_next::home_dir().map(|h| h.join("Library/Inform/Extensions"));
    #[cfg(target_os = "windows")]
    return dirs_next::document_dir().map(|d| d.join("Inform/Extensions"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    return dirs_next::home_dir().map(|h| h.join("Inform/Extensions"));
}

fn version_ord(v: &str) -> (u64, u64, u64) {
    let parts: Vec<u64> = v.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}
