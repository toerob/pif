use ansi_term::Colour::*;
use std::{fs::{self, File}, io::{Cursor, Write}, path::Path, process::exit};
use std::path::PathBuf;

use crate::{
    args::{Color, GlobalOptions, InstallOptions, InteractiveFictionSystem},
    color::{print_success_msg, print_warning_msg},
    detect::detect_system,
    gitops::clone_or_pull_repo,
    list::system_to_dir,
    makefile::add_make_file_entry,
    model::{load_registry, BuildEntry, LoadedRelease},
    config::{expand_path, load_config, version_matches_any, VersionSpec},
    update::{get_registry_root, update_extensions},
    db::{self, get_or_create_table, record_installation},
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

    let config = load_config();

    for (req_name, req_version) in &requests {
        let found = entries.iter().find(|e| {
            e.package.id.to_lowercase() == *req_name
                || e.package.name.to_lowercase() == *req_name
        });

        let entry = match found {
            Some(e) => e,
            None => {
                print_warning_msg(use_colours, format!("No extension found for '{}'\n", req_name));
                continue;
            }
        };

        // When resolving "latest", restrict candidates to configured version specs.
        // An explicit version pin bypasses the filter.
        let filtered_releases: Vec<LoadedRelease>;
        let releases_slice = if req_version == "latest" || req_version.is_empty() {
            if let Some(specs_raw) = config.system_versions.get(&entry.system) {
                let specs: Vec<VersionSpec> = specs_raw.iter().map(|s| VersionSpec::parse(s)).collect();
                filtered_releases = entry.releases.iter()
                    .filter(|r| version_matches_any(&r.version, &specs))
                    .cloned()
                    .collect();
                &filtered_releases
            } else {
                &entry.releases
            }
        } else {
            &entry.releases
        };

        let release = resolve_version(releases_slice, req_version);
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

        let library_path = if let Some(ref p) = explicit_library_path {
            p.clone()
        } else if let Some(dir) = config.install_dirs.get(&entry.system) {
            let p = expand_path(dir);
            if !p.exists() {
                if let Err(e) = fs::create_dir_all(&p) {
                    print_warning_msg(use_colours, format!(
                        "Could not create configured install dir {}: {}\n", p.display(), e
                    ));
                    continue;
                }
            }
            p
        } else if is_inform {
            inform_extensions_dir().unwrap_or_else(|| PathBuf::from("."))
        } else {
            PathBuf::from(".")
        };

        // git clones into its own named dir; zip and raw files go straight to
        // library_path so they supply their own structure / filename.
        let install_path = if is_inform {
            library_path.join(&entry.package.author)
        } else if source.format == "git" {
            library_path.join(&entry.package.name)
        } else {
            library_path.clone()
        };

        if !install_path.exists() {
            if let Err(e) = fs::create_dir_all(&install_path) {
                print_warning_msg(use_colours, format!("Could not create {}: {}\n", install_path.display(), e));
                continue;
            }
        }

        let install_path_str = dunce::canonicalize(&install_path)
            .unwrap_or_else(|_| install_path.clone())
            .to_string_lossy()
            .into_owned();

        // For non-git installs, skip if the db already has this name+path+version.
        if source.format != "git" {
            if let Ok(conn) = get_or_create_table() {
                if db::is_installed(&conn, &entry.package.name, &install_path_str, &loaded.version) {
                    println!("Already installed.");
                    continue;
                }
            }
        }

        // None = failed, Some(true) = installed/updated, Some(false) = already up to date
        let outcome: Option<bool> = match source.format.as_str() {
            "zip" => install_zip(&source.url, &install_path, false, use_colours).then_some(true),
            "git" => install_git(&source.url, source.branch.as_deref(), &install_path, use_colours),
            fmt   => install_raw_file(
                &source.url, &install_path, fmt,
                Some(&entry.package.name),
                use_colours,
            ).then_some(true),
        };

        match outcome {
            None => continue,
            Some(false) => continue,
            Some(true) => {}
        }

        record_entry(&entry.package.name, &install_path_str, &loaded.version, use_colours);
        print_success_msg(use_colours, format!(
            " ==> {} v{} [{}] installed into {}\n", entry.package.name, loaded.version,
            entry.system,
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

fn install_git(url: &str, branch: Option<&str>, dest: &Path, use_colours: bool) -> Option<bool> {
    let branch = branch.unwrap_or("master");
    clone_or_pull_repo(url, branch, &dest.to_path_buf())
        .map_err(|e| print_warning_msg(use_colours, format!("Git error: {}\n", e)))
        .ok()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{LoadedRelease, Release};

    fn make_release(version: &str) -> LoadedRelease {
        LoadedRelease {
            version: version.to_string(),
            release: Release {
                schema_version: 1,
                maintainer: None, channel: None, date: None, description: None,
                compatibility: None, dependencies: None, source: None, build: None,
            },
        }
    }

    // ── to_raw_github_url ─────────────────────────────────────────────────────

    #[test]
    fn raw_url_converts_blob() {
        let input    = "https://github.com/owner/repo/blob/main/path/to/file.h";
        let expected = "https://raw.githubusercontent.com/owner/repo/main/path/to/file.h";
        assert_eq!(to_raw_github_url(input), expected);
    }

    #[test]
    fn raw_url_leaves_non_blob_unchanged() {
        let url = "https://example.com/file.zip";
        assert_eq!(to_raw_github_url(url), url);
    }

    #[test]
    fn raw_url_leaves_raw_url_unchanged() {
        let url = "https://raw.githubusercontent.com/owner/repo/main/file.h";
        assert_eq!(to_raw_github_url(url), url);
    }

    // ── build_entries_to_flags ────────────────────────────────────────────────

    #[test]
    fn flags_lib_and_source() {
        let entries = vec![
            BuildEntry { kind: "lib".into(),    path: Some("adv3/adv3.tl".into()), value: None },
            BuildEntry { kind: "source".into(), path: Some("src/main.t".into()),   value: None },
        ];
        assert_eq!(
            build_entries_to_flags(&entries),
            vec!["-lib adv3/adv3.tl", "-source src/main.t"]
        );
    }

    #[test]
    fn flags_define() {
        let entries = vec![
            BuildEntry { kind: "define".into(), path: None, value: Some("USE_HTML".into()) },
        ];
        assert_eq!(build_entries_to_flags(&entries), vec!["-D USE_HTML"]);
    }

    #[test]
    fn flags_unknown_kind_skipped() {
        let entries = vec![
            BuildEntry { kind: "unknown".into(), path: Some("x".into()), value: None },
        ];
        assert!(build_entries_to_flags(&entries).is_empty());
    }

    // ── version_ord ───────────────────────────────────────────────────────────

    #[test]
    fn version_ord_semver() {
        assert!(version_ord("2.1.0") > version_ord("2.0.9"));
        assert!(version_ord("1.0.0") < version_ord("1.0.1"));
        assert_eq!(version_ord("1.2.3"), (1, 2, 3));
    }

    #[test]
    fn version_ord_single_integer() {
        assert_eq!(version_ord("16"), (16, 0, 0));
        assert!(version_ord("16") > version_ord("9"));
    }

    #[test]
    fn version_ord_unparseable_is_zero() {
        assert_eq!(version_ord(""), (0, 0, 0));
        assert_eq!(version_ord("abc"), (0, 0, 0));
    }

    // ── resolve_version ───────────────────────────────────────────────────────

    #[test]
    fn resolve_latest_picks_highest() {
        let releases = vec![make_release("1.0.0"), make_release("2.0.0"), make_release("1.5.0")];
        assert_eq!(resolve_version(&releases, "latest").unwrap().version, "2.0.0");
    }

    #[test]
    fn resolve_exact_version() {
        let releases = vec![make_release("1.0.0"), make_release("2.0.0")];
        assert_eq!(resolve_version(&releases, "1.0.0").unwrap().version, "1.0.0");
    }

    #[test]
    fn resolve_missing_version_returns_none() {
        let releases = vec![make_release("1.0.0")];
        assert!(resolve_version(&releases, "9.9.9").is_none());
    }

    #[test]
    fn resolve_empty_string_acts_as_latest() {
        let releases = vec![make_release("1.0.0"), make_release("3.0.0")];
        assert_eq!(resolve_version(&releases, "").unwrap().version, "3.0.0");
    }
}
