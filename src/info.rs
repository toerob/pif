use ansi_term::Colour::*;
use std::path::Path;
use sublime_fuzzy::FuzzySearch;

use crate::{
    args::{Color, GlobalOptions, InfoCommand, InteractiveFictionSystem},
    color::{create_info_message, create_success_msg},
    db::{get_installations_by_name, get_or_create_table},
    detect::detect_system,
    list::system_to_dir,
    model::{load_registry, PackageEntry},
    update::{get_registry_root, update_extensions},
};

pub fn extensions_info(
    cmd: &InfoCommand,
    global_options: &GlobalOptions,
    update_needed: bool,
) {
    if update_needed {
        update_extensions(global_options);
    }

    let names = &cmd.name;

    if names.is_empty() && cmd.author.is_none() && cmd.keyword.is_none() && cmd.tag.is_none() {
        println!(
            "{}",
            Red.paint("No packages specified. Example: pif info smarter-parser")
        );
        return;
    }

    let use_colors = global_options.color != Color::Never;

    let detected_system = if global_options.system == InteractiveFictionSystem::Auto {
        detect_system().0
    } else {
        global_options.system.clone()
    };

    let registry_root = get_registry_root();

    let entries = match load_registry(&registry_root, None) {
        Ok(e) => e,
        Err(e) => { eprintln!("Could not load registry: {}", e); return; }
    };

    let lowercase_names: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();

    let mut matches: Vec<&PackageEntry> = entries.iter().filter(|e| {
        if lowercase_names.is_empty() {
            return true;
        }
        let id   = e.package.id.to_lowercase();
        let name = e.package.name.to_lowercase();
        lowercase_names.iter().any(|q| id == *q || name == *q)
    }).collect();

    if let Some(author) = &cmd.author {
        let q = author.to_lowercase();
        matches.retain(|e| {
            FuzzySearch::new(&q, &e.package.author.to_lowercase())
                .case_insensitive().best_match().is_some()
        });
    }
    if let Some(keyword) = &cmd.keyword {
        let q = keyword.to_lowercase();
        matches.retain(|e| {
            FuzzySearch::new(&q, &e.package.name.to_lowercase())
                .case_insensitive().best_match().is_some()
        });
    }
    if let Some(tag) = &cmd.tag {
        let q = tag.to_lowercase();
        matches.retain(|e| {
            e.package.tags.as_deref().unwrap_or(&[])
                .iter().any(|t| t.to_lowercase() == q)
        });
    }

    if matches.is_empty() {
        println!("{}", Red.paint(format!("No extension found for: {}", names.join(", "))));
        return;
    }

    if detected_system != InteractiveFictionSystem::Unknown {
        println!("{}\n", Yellow.paint(format!("[Detected system: {:?}]", detected_system)));
    }

    let conn = get_or_create_table().ok();

    for entry in matches {
        let detected_dir = system_to_dir(&detected_system);
        if detected_dir.is_some() && detected_dir.map_or(false, |d| d != entry.system) {
            println!("{}", Yellow.paint(format!(
                "Note: '{}' is a {} extension, not compatible with the detected system ({:?}).",
                entry.package.name, entry.system, detected_system
            )));
        }

        let name_str = create_success_msg(use_colors, entry.package.name.clone());
        let desc = entry.package.description.as_deref().unwrap_or("");
        println!("{} by {}\n{}\n", name_str, entry.package.author, desc);

        if entry.releases.is_empty() {
            continue;
        }

        // Versions
        println!("Available versions:");
        let mut releases = entry.releases.clone();
        releases.sort_by(|a, b| version_ord(&a.version).cmp(&version_ord(&b.version)));
        let total = releases.len();

        let latest_idx = releases.iter().rposition(|r| r.version.to_uppercase() == "SNAPSHOT")
            .unwrap_or(total - 1);

        for (i, loaded) in releases.iter().enumerate() {
            let ver_str = create_info_message(use_colors, loaded.version.clone());
            let url = loaded.release.source.as_ref()
                .map(|s| s.url.as_str())
                .unwrap_or("(no url)");
            let date = loaded.release.date.as_deref().unwrap_or("-");
            let latest = if i == latest_idx {
                Green.paint(" <== LATEST").to_string()
            } else {
                String::new()
            };
            println!("  * {} {}  ({}){}", ver_str, url, date, latest);
        }

        // Dependencies from latest release
        if let Some(latest) = releases.last() {
            if let Some(deps) = &latest.release.dependencies {
                if !deps.is_empty() {
                    println!("\nDependencies:");
                    for dep in deps {
                        match &dep.constraint {
                            Some(c) => println!("  * {} ({})", dep.id, c),
                            None    => println!("  * {}", dep.id),
                        }
                    }
                }
            }
        }

        // Installation status
        if let Some(ref conn) = conn {
            if let Ok(installs) = get_installations_by_name(conn, &entry.package.name) {
                if !installs.is_empty() {
                    println!("\nInstalled:");
                    for inst in &installs {
                        let missing = if !Path::new(&inst.path).exists() { "  [missing]" } else { "" };
                        let date = inst.installed_at.as_deref()
                            .map(|d| format!("  installed {}", d))
                            .unwrap_or_default();
                        let ver_prefix = inst.version.as_deref()
                            .map(|v| format!("{}  ", create_info_message(use_colors, v.to_string())))
                            .unwrap_or_default();
                        println!("  {}{}{}{}", ver_prefix, inst.path, date, missing);
                    }
                }
            }
        }

        println!();
    }
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

    #[test]
    fn version_ord_full_semver() {
        assert_eq!(version_ord("1.2.3"), (1, 2, 3));
    }

    #[test]
    fn version_ord_missing_patch() {
        assert_eq!(version_ord("1.2"), (1, 2, 0));
    }

    #[test]
    fn version_ord_major_only() {
        assert_eq!(version_ord("3"), (3, 0, 0));
    }

    #[test]
    fn version_ord_empty_string() {
        assert_eq!(version_ord(""), (0, 0, 0));
    }

    #[test]
    fn version_ord_non_numeric_parts_treated_as_zero() {
        assert_eq!(version_ord("SNAPSHOT"), (0, 0, 0));
    }

    #[test]
    fn version_ord_ordering() {
        assert!(version_ord("1.10.0") > version_ord("1.9.0"));
        assert!(version_ord("2.0.0") > version_ord("1.99.99"));
        assert!(version_ord("1.2.3") < version_ord("1.2.4"));
    }
}
