use std::fs;

use ansi_term::Colour::*;
use chrono::Local;
use dialoguer::{Confirm, Input};

use crate::{
    args::{Color, GlobalOptions, InteractiveFictionSystem},
    color::{print_success_msg, print_warning_msg},
    detect::detect_system,
    gitops::latest_semver_tag,
    list::system_to_dir,
    model::{Build, BuildEntry, Package, Release, Source},
    update::{get_registry_root, load_cached_registry_location, update_extensions},
};

// ── Prompt helpers ────────────────────────────────────────────────────────────

fn ask(label: &str, default: Option<&str>) -> String {
    Input::<String>::new()
        .with_prompt(label)
        .default(default.unwrap_or("").to_string())
        .interact_text()
        .unwrap_or_default()
}

fn ask_required(label: &str, default: Option<&str>) -> String {
    let mut b = Input::<String>::new()
        .with_prompt(label)
        .validate_with(|s: &String| -> Result<(), &str> {
            if s.is_empty() { Err("Required") } else { Ok(()) }
        });
    if let Some(d) = default.filter(|d| !d.is_empty()) {
        b = b.default(d.to_string());
    }
    b.interact_text().unwrap()
}

// ── Misc helpers ──────────────────────────────────────────────────────────────

fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn to_ssh_url(url: &str) -> String {
    url.strip_prefix("https://github.com/")
        .map(|path| format!("git@github.com:{}", path.trim_end_matches('/')))
        .unwrap_or_else(|| url.to_string())
}

fn push_branch(repo: &git2::Repository, branch_name: &str, registry_url: &str) -> Result<(), String> {
    let workdir = repo.workdir().ok_or("no working directory")?;
    let push_url = to_ssh_url(registry_url);
    let output = std::process::Command::new("git")
        .args(["push", "--force", &push_url, branch_name])
        .current_dir(workdir)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn pr_compare_url(repo_url: &str, base: &str, branch: &str) -> String {
    let encoded = branch.replace('/', "%2F");
    format!("{}/compare/{}...{}?expand=1", repo_url.trim_end_matches(".git"), base, encoded)
}

fn parse_build_entries(raw: &str) -> Vec<BuildEntry> {
    raw.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|entry| {
            let mut tokens = entry.splitn(2, char::is_whitespace);
            let flag = tokens.next()?;
            let arg  = tokens.next().map(str::trim);
            match (flag, arg) {
                ("-lib",    Some(p)) => Some(BuildEntry { kind: "lib".into(),    path: Some(p.into()), value: None }),
                ("-source", Some(p)) => Some(BuildEntry { kind: "source".into(), path: Some(p.into()), value: None }),
                ("-D",      Some(v)) => Some(BuildEntry { kind: "define".into(), path: None, value: Some(v.into()) }),
                _ => None,
            }
        })
        .collect()
}

fn slugify(s: &str) -> String {
    deunicode::deunicode(s)
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    // parse_build_entries
    #[test]
    fn parse_build_entries_all_three_kinds() {
        let entries = parse_build_entries("-lib foo.tl; -source bar.t; -D MYFLAG=1");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].kind, "lib");
        assert_eq!(entries[0].path, Some("foo.tl".into()));
        assert_eq!(entries[1].kind, "source");
        assert_eq!(entries[1].path, Some("bar.t".into()));
        assert_eq!(entries[2].kind, "define");
        assert_eq!(entries[2].value, Some("MYFLAG=1".into()));
    }

    #[test]
    fn parse_build_entries_empty_string() {
        assert!(parse_build_entries("").is_empty());
    }

    #[test]
    fn parse_build_entries_whitespace_only_segments() {
        assert!(parse_build_entries("  ;  ; ").is_empty());
    }

    #[test]
    fn parse_build_entries_unknown_flag_skipped() {
        let entries = parse_build_entries("-other foo; -lib bar.tl");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "lib");
    }

    // slugify
    #[test]
    fn slugify_unicode_normalization() {
        assert_eq!(slugify("Héllo Wörld"), "hello-world");
    }

    #[test]
    fn slugify_consecutive_separators_collapsed() {
        assert_eq!(slugify("foo  bar"), "foo-bar");
    }

    #[test]
    fn slugify_leading_trailing_dashes_removed() {
        assert_eq!(slugify("!foo!"), "foo");
    }

    #[test]
    fn slugify_already_clean() {
        assert_eq!(slugify("my-extension"), "my-extension");
    }

    // to_ssh_url
    #[test]
    fn to_ssh_url_converts_github_https() {
        assert_eq!(to_ssh_url("https://github.com/user/repo"), "git@github.com:user/repo");
    }

    #[test]
    fn to_ssh_url_strips_trailing_slash() {
        assert_eq!(to_ssh_url("https://github.com/user/repo/"), "git@github.com:user/repo");
    }

    #[test]
    fn to_ssh_url_non_github_passthrough() {
        let url = "https://gitlab.com/user/repo";
        assert_eq!(to_ssh_url(url), url);
    }

    // pr_compare_url
    #[test]
    fn pr_compare_url_simple_branch() {
        let url = pr_compare_url("https://github.com/user/repo", "main", "feature");
        assert_eq!(url, "https://github.com/user/repo/compare/main...feature?expand=1");
    }

    #[test]
    fn pr_compare_url_branch_with_slashes_encoded() {
        let url = pr_compare_url("https://github.com/user/repo", "main", "pif/add-foo-1.0.0");
        assert!(url.contains("pif%2Fadd-foo-1.0.0"));
    }

    #[test]
    fn pr_compare_url_strips_git_suffix() {
        let url = pr_compare_url("https://github.com/user/repo.git", "main", "feature");
        assert_eq!(url, "https://github.com/user/repo/compare/main...feature?expand=1");
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

pub fn publish_extension(dir: &str, global_options: &GlobalOptions) {
    let use_colours = Color::Never != global_options.color;

    update_extensions(global_options);

    let registry_root = get_registry_root();
    let repo_dir = registry_root.parent()
        .expect("Registry root has no parent")
        .to_path_buf();

    // Detect or ask for system
    let (detected_system, _) = detect_system();
    let system = if matches!(
        detected_system,
        InteractiveFictionSystem::Unknown | InteractiveFictionSystem::Auto
    ) {
        println!("Could not auto-detect IF system.");
        let s = ask_required("System (tads3 / dialog / inform / inform6)", None);
        match s.to_lowercase().as_str() {
            "tads3"   => InteractiveFictionSystem::Tads3,
            "dialog"  => InteractiveFictionSystem::Dialog,
            "inform"  => InteractiveFictionSystem::Inform,
            "inform6" => InteractiveFictionSystem::Inform6,
            other => { eprintln!("Unknown system '{}'. Aborting.", other); return; }
        }
    } else {
        detected_system
    };

    let system_dir = match system_to_dir(&system) {
        Some(d) => d,
        None => { eprintln!("Unsupported system. Aborting."); return; }
    };

    // Git context from the extension directory
    let target_dir = fs::canonicalize(dir)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let local_repo = git2::Repository::discover(&target_dir).ok();
    let default_name = target_dir.file_name()
        .and_then(|n| n.to_str()).unwrap_or("").to_string();
    let default_url = local_repo.as_ref()
        .and_then(|r| r.find_remote("origin").ok())
        .and_then(|r| r.url().map(String::from));
    let default_branch = local_repo.as_ref()
        .and_then(|r| r.head().ok())
        .and_then(|h| h.shorthand().map(String::from));
    let default_version = latest_semver_tag(&target_dir);
    let git_config = git2::Config::open_default().ok();
    let default_author = git_config.as_ref()
        .and_then(|c| c.get_string("user.name").ok());
    let default_email = git_config.as_ref()
        .and_then(|c| c.get_string("user.email").ok());

    println!("\n{}", Yellow.paint("=== pif publish ==="));

    // Package prompts
    let name      = ask_required("Extension name",     Some(&default_name));
    let author    = ask_required("Author",              default_author.as_deref());
    let desc_raw  = ask("Description (optional)",       None);
    let id        = ask_required("Package ID (lowercase, hyphens only)", Some(&slugify(&name)));
    let namespace = ask_required("Namespace (author or category)",      Some(&slugify(&author)));

    // Release prompts
    let version   = ask_required("Version (semver, e.g. 1.0.0)", default_version.as_deref());
    let url       = ask_required("Source URL",                   default_url.as_deref());

    let fmt_default = if url.ends_with(".git") {
        "git"
    } else {
        url.rsplit('.').next().unwrap_or("zip")
    };
    let format = ask_required("Format (zip / git / h / t / ...)", Some(fmt_default));

    let git_branch = if format == "git" {
        println!("{}", Yellow.paint("Note: the repository and branch must be publicly accessible."));
        let b = ask("Git branch", default_branch.as_deref().or(Some("master")));
        (!b.is_empty()).then_some(b)
    } else {
        None
    };

    let inform_branch = if system == InteractiveFictionSystem::Inform {
        let example = format!("v{}-i10.1.yaml", version);
        Some(ask_required(&format!("Inform compatible version (e.g. 10.1, 9.3 — gives {})", example), None))
    } else {
        None
    };

    let date_input = ask("Release date (YYYY-MM-DD)", Some(&today_iso()));
    let date = (!date_input.is_empty()).then_some(date_input);

    // TADS3 build entries
    let build = if system == InteractiveFictionSystem::Tads3 {
        let raw = ask("Build entries, semicolon-separated (e.g. -lib library.tl; -source file.t; -D MYFLAG)", None);
        let exports = parse_build_entries(&raw);
        if exports.is_empty() { None } else {
            Some(Build { exports: Some(exports), private: None })
        }
    } else {
        None
    };

    // Resolve file paths
    let pkg_dir          = registry_root.join(system_dir).join(&namespace).join(&id);
    let releases_dir     = pkg_dir.join("releases");
    let version_bare = version.trim_start_matches('v');
    let release_filename = match &inform_branch {
        Some(b) => format!("v{}-i{}.yaml", version_bare, b),
        None    => format!("v{}.yaml", version_bare),
    };
    let package_path  = pkg_dir.join("package.yaml");
    let release_path  = releases_dir.join(&release_filename);
    let package_exists = package_path.exists();
    let release_exists = release_path.exists();

    // Serialize
    let package = Package {
        schema_version: 1,
        id:             id.clone(),
        name:           name.clone(),
        author:         author.clone(),
        description:    (!desc_raw.is_empty()).then_some(desc_raw),
        tags:           None,
    };
    let release = Release {
        schema_version: 1,
        maintainer:     None,
        channel:        None,
        date,
        description:    None,
        compatibility:  None,
        dependencies:   None,
        source:         Some(Source { url, format, branch: git_branch }),
        build,
    };

    let package_yaml = serde_yaml::to_string(&package).unwrap();
    let release_yaml = serde_yaml::to_string(&release).unwrap();

    // Preview
    println!();
    if package_exists {
        println!("{}", Yellow.paint("package.yaml already exists — will not overwrite."));
    } else {
        println!("{}\n{}", Yellow.paint("New package.yaml:"), package_yaml);
    }
    if release_exists {
        print_warning_msg(use_colours, format!("Release {} already exists and will be overwritten.\n", release_filename));
    }
    println!("{} {}\n{}", Yellow.paint("New release:"), release_filename, release_yaml);

    if !Confirm::new()
        .with_prompt("Write these files and open a PR?")
        .default(true)
        .interact()
        .unwrap_or(false)
    {
        println!("Aborted.");
        return;
    }

    // Write files
    if let Err(e) = fs::create_dir_all(&releases_dir) {
        eprintln!("Could not create directories: {}", e); return;
    }
    if !package_exists {
        if let Err(e) = fs::write(&package_path, &package_yaml) {
            eprintln!("Could not write package.yaml: {}", e); return;
        }
    }
    if let Err(e) = fs::write(&release_path, &release_yaml) {
        eprintln!("Could not write release file: {}", e); return;
    }

    // Commit on a feature branch
    let branch_name = format!(
        "pif/add-{}-{}",
        id.replace(|c: char| !c.is_alphanumeric() && c != '-', "-"),
        version.replace(|c: char| !c.is_alphanumeric() && c != '-', "-"),
    );

    let index_repo = match git2::Repository::open(&repo_dir) {
        Ok(r)  => r,
        Err(e) => { eprintln!("Could not open registry repo: {}", e); return; }
    };

    let head_commit = match index_repo.head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| index_repo.find_commit(oid).ok())
    {
        Some(c) => c,
        None    => { eprintln!("Could not read HEAD commit."); return; }
    };

    let _ = index_repo.branch(&branch_name, &head_commit, false);

    let mut git_index = index_repo.index().unwrap();
    let rel_package = package_path.strip_prefix(&repo_dir).unwrap().to_path_buf();
    let rel_release = release_path.strip_prefix(&repo_dir).unwrap().to_path_buf();
    git_index.add_path(&rel_package).unwrap();
    git_index.add_path(&rel_release).unwrap();
    git_index.write().unwrap();

    let tree     = index_repo.find_tree(git_index.write_tree().unwrap()).unwrap();
    let sig      = index_repo.signature().unwrap_or_else(|_| {
        git2::Signature::now(
            &author,
            default_email.as_deref().unwrap_or("pif@localhost"),
        ).unwrap()
    });

    if let Err(e) = index_repo.commit(
        Some(&format!("refs/heads/{}", branch_name)),
        &sig, &sig,
        &format!("Add {} {}", name, version),
        &tree,
        &[&head_commit],
    ) {
        eprintln!("Could not create commit: {}", e); return;
    }

    // Checkout the new branch so the working tree is on it
    let branch_ref = index_repo.revparse_single(&format!("refs/heads/{}", branch_name)).unwrap();
    index_repo.checkout_tree(&branch_ref, Some(git2::build::CheckoutBuilder::default().force())).unwrap();
    index_repo.set_head(&format!("refs/heads/{}", branch_name)).unwrap();

    // Push and open PR
    let loc = match load_cached_registry_location() {
        Some(l) => l,
        None => {
            print_warning_msg(use_colours, "No cached registry location.\n".into());
            return;
        }
    };
    let base_branch = loc.branch.as_deref().unwrap_or("main");

    match push_branch(&index_repo, &branch_name, &loc.url) {
        Ok(_) => {
            let link = pr_compare_url(&loc.url, base_branch, &branch_name);
            print_success_msg(use_colours, "\nBranch pushed. Opening PR form...\n".to_string());
            let _ = open::that(&link);
            println!("  {}\n", Green.paint(&link));
        }
        Err(e) => {
            print_warning_msg(use_colours, format!("\nCould not push ({})\n", e));
            println!("Commit is ready locally on branch '{}'.", branch_name);
            println!("Push manually and open a PR against '{}'.", base_branch);
        }
    }
}
