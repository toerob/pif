use ansi_term::Colour::*;
use std::{fs, path::{Path, PathBuf}, process::Command};

use dialoguer::Confirm;

use crate::{
    args::{Color, GlobalOptions},
    color::print_warning_msg,
    gitops::{clone_or_pull_repo, get_or_create_repo_dir},
    model::RegistryLocation,
};

const INDEX_URL: &str =
    "https://raw.githubusercontent.com/toerob/pif-index/main/registry-location.yaml";

pub fn update_extensions(global_options: &GlobalOptions) {
    let use_colour = Color::Never != global_options.color;

    println!("{}", Yellow.paint("Fetching registry location..."));

    let location = match fetch_registry_location() {
        Ok(loc) => {
            if let Some(msg) = &loc.message {
                if !msg.is_empty() {
                    println!("{}", Yellow.paint(format!("Notice: {}", msg)));
                }
            }
            cache_registry_location(&loc);
            loc
        }
        Err(e) => {
            match e {
                FetchError::Network(msg) => {
                    print_warning_msg(use_colour, format!(
                        "Network error fetching registry location: {}. Falling back to cached location.\n", msg
                    ));
                }
                FetchError::Upgrade(msg) => {
                    print_warning_msg(use_colour, format!(
                        "Could not load registry index: {}.\n", msg
                    ));
                    println!("This may mean the registry has moved. Please upgrade pif:");
                    println!("  brew upgrade pif");
                }
            }
            match load_cached_registry_location() {
                Some(loc) => loc,
                None => {
                    print_warning_msg(use_colour, "No cached location available. Cannot update.\n".into());
                    return;
                }
            }
        }
    };

    let branch = location.branch.as_deref().unwrap_or("main");

    println!("{}", Yellow.paint(format!("Updating registry from {}...", location.url)));

    let repo_dir = dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo");

    match get_or_create_repo_dir(repo_dir.to_str().unwrap()) {
        Ok(repo_path) => {
            if repo_path.join(".git").exists() {
                match pull_registry(&repo_path, &location.url, branch, use_colour) {
                    Ok(true)  => println!("{}", Green.paint("Registry updated.")),
                    Ok(false) => println!("{}", Green.paint("Registry already up to date.")),
                    Err(_)    => {}
                }
            } else {
                match clone_or_pull_repo(&location.url, branch, &repo_path) {
                    Ok(_)  => println!("{}", Green.paint("Registry cloned.")),
                    Err(e) => print_warning_msg(use_colour, format!("Error: {}\n", e)),
                }
            }
        }
        Err(e) => eprintln!("Failed to create repository directory: {}", e),
    }
}

fn git_run(args: &[&str], dir: &Path) -> Result<String, String> {
    let out = Command::new("git").args(args).current_dir(dir).output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn pull_registry(repo_path: &Path, url: &str, branch: &str, use_colour: bool) -> Result<bool, ()> {
    let head_before = git_run(&["rev-parse", "HEAD"], repo_path).unwrap_or_default();

    if let Err(e) = git_run(&["fetch", "--all", "--prune"], repo_path) {
        print_warning_msg(use_colour, format!("Fetch failed: {}\n", e));
        return prompt_fresh_clone(repo_path, url, branch, use_colour);
    }

    // Prefer the upstream tracking ref; fall back to origin/{branch}.
    let upstream = git_run(&["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"], repo_path)
        .unwrap_or_else(|_| format!("origin/{}", branch));

    // Detect force push: local HEAD is no longer an ancestor of the remote.
    let diverged = git_run(&["merge-base", "--is-ancestor", "HEAD", &upstream], repo_path).is_err();
    if diverged {
        println!("{}", Yellow.paint("Remote history has changed (force push). Resetting local registry..."));
    }

    match git_run(&["reset", "--hard", &upstream], repo_path) {
        Ok(_) => {}
        Err(e) => {
            print_warning_msg(use_colour, format!("Reset failed: {}\n", e));
            // After a force push the repo may be in a broken state — re-clone automatically.
            if diverged {
                println!("{}", Yellow.paint("Re-cloning registry from scratch..."));
                return auto_fresh_clone(repo_path, url, branch, use_colour);
            }
            return prompt_fresh_clone(repo_path, url, branch, use_colour);
        }
    }

    let head_after = git_run(&["rev-parse", "HEAD"], repo_path).unwrap_or_default();
    Ok(head_before != head_after)
}

fn auto_fresh_clone(repo_path: &Path, url: &str, branch: &str, use_colour: bool) -> Result<bool, ()> {
    if let Err(e) = fs::remove_dir_all(repo_path) {
        print_warning_msg(use_colour, format!("Could not remove registry cache: {}\n", e));
        return Err(());
    }
    if let Err(e) = fs::create_dir_all(repo_path) {
        print_warning_msg(use_colour, format!("Could not recreate directory: {}\n", e));
        return Err(());
    }
    match clone_or_pull_repo(url, branch, &repo_path.to_path_buf()) {
        Ok(_)  => { println!("{}", Green.paint("Registry re-cloned successfully.")); Ok(true) }
        Err(e) => { print_warning_msg(use_colour, format!("Re-clone failed: {}\n", e)); Err(()) }
    }
}

fn prompt_fresh_clone(repo_path: &Path, url: &str, branch: &str, use_colour: bool) -> Result<bool, ()> {
    let confirmed = Confirm::new()
        .with_prompt("Reset the local registry cache and re-clone from scratch?")
        .default(false)
        .interact()
        .unwrap_or(false);

    if !confirmed {
        return Err(());
    }

    if let Err(e) = fs::remove_dir_all(repo_path) {
        print_warning_msg(use_colour, format!("Could not remove registry cache: {}\n", e));
        return Err(());
    }
    if let Err(e) = fs::create_dir_all(repo_path) {
        print_warning_msg(use_colour, format!("Could not recreate directory: {}\n", e));
        return Err(());
    }

    match clone_or_pull_repo(url, branch, &repo_path.to_path_buf()) {
        Ok(_)  => { println!("{}", Green.paint("Registry re-cloned successfully.")); Ok(true) }
        Err(e) => { print_warning_msg(use_colour, format!("Re-clone failed: {}\n", e)); Err(()) }
    }
}

/// Returns the local path to the registry directory, reading the cached root from
/// the fetched registry-location.yaml. Falls back to "registry" if not available.
pub fn get_registry_root() -> PathBuf {
    let root = load_cached_registry_location()
        .map(|l| l.root)
        .unwrap_or_else(|| "registry".to_string());

    dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo")
        .join(root)
}

enum FetchError {
    Network(String),
    Upgrade(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Network(e) => write!(f, "{}", e),
            FetchError::Upgrade(e) => write!(f, "{}", e),
        }
    }
}

fn fetch_registry_location() -> Result<RegistryLocation, FetchError> {
    let response = reqwest::blocking::get(INDEX_URL).map_err(|e| {
        if e.is_connect() || e.is_timeout() {
            FetchError::Network(e.to_string())
        } else {
            FetchError::Upgrade(e.to_string())
        }
    })?;

    if !response.status().is_success() {
        return Err(FetchError::Upgrade(format!(
            "index returned HTTP {}", response.status()
        )));
    }

    let text = response.text().map_err(|e| FetchError::Network(e.to_string()))?;

    serde_yaml::from_str::<RegistryLocation>(&text)
        .map_err(|e| FetchError::Upgrade(format!("could not parse registry location: {}", e)))
}

fn cache_path() -> PathBuf {
    dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("registry-location.yaml")
}

fn cache_registry_location(loc: &RegistryLocation) {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let content = format!(
        "version: {}\nurl: {}\nroot: {}\nbranch: {}\n",
        loc.version,
        loc.url,
        loc.root,
        loc.branch.as_deref().unwrap_or("main"),
    );
    if let Err(e) = fs::write(&path, content) {
        eprintln!("Warning: could not cache registry location: {}", e);
    }
}

pub fn load_cached_registry_location() -> Option<RegistryLocation> {
    let text = fs::read_to_string(cache_path()).ok()?;
    serde_yaml::from_str(&text).ok()
}
