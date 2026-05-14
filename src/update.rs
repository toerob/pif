use ansi_term::Colour::*;
use std::{fs, path::PathBuf};

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
            if let Err(e) = clone_or_pull_repo(&location.url, branch, &repo_path) {
                print_warning_msg(use_colour, format!("Error: {}\n", e));
            } else {
                println!("{}", Green.paint("Registry updated."));
            }
        }
        Err(e) => eprintln!("Failed to create repository directory: {}", e),
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
