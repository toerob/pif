use ansi_term::Colour::*;
use sublime_fuzzy::FuzzySearch;

use crate::{
    args::{Color, GlobalOptions, InteractiveFictionSystem, ListOptions, ListPresentation, SortProperty},
    detect::detect_system,
    model::{load_registry, PackageEntry},
    update::{get_registry_root, update_extensions},
};

pub fn list_extensions(
    list_options: &ListOptions,
    global_options: &GlobalOptions,
    update_needed: bool,
) {
    if update_needed {
        update_extensions(global_options);
    }

    let system_type = if global_options.system == InteractiveFictionSystem::Auto {
        detect_system().0
    } else {
        global_options.system.clone()
    };

    println!("{}", Yellow.paint(format!("System: {:?}", system_type)));

    let registry_root = get_registry_root();

    let system_filter = system_to_dir(&system_type);

    let mut entries = match load_registry(&registry_root, system_filter) {
        Ok(e) => e,
        Err(e) => { eprintln!("Could not load registry: {}", e); return; }
    };

    if let Some(author) = &list_options.author {
        let q = author.to_lowercase();
        entries.retain(|e| {
            FuzzySearch::new(&q, &e.package.author.to_lowercase())
                .case_insensitive().best_match().is_some()
        });
    }
    if let Some(keyword) = &list_options.keyword {
        let q = keyword.to_lowercase();
        entries.retain(|e| {
            FuzzySearch::new(&q, &e.package.name.to_lowercase())
                .case_insensitive().best_match().is_some()
        });
    }

    match list_options.sort_property {
        SortProperty::Name   => entries.sort_by(|a, b| a.package.name.cmp(&b.package.name)),
        SortProperty::Author => entries.sort_by(|a, b| a.package.author.cmp(&b.package.author)),
        SortProperty::Date   => entries.sort_by_key(|e| {
            e.releases.iter()
                .filter_map(|r| r.release.date.clone())
                .max()
                .unwrap_or_default()
        }),
    }

    let delimiter = if list_options.presentation == ListPresentation::Comma { "," } else { "\n" };
    let lines: Vec<_> = entries.iter().map(|e| present(e, global_options)).collect();
    println!("{}", lines.join(delimiter));
    println!();
    println!("[Filter by -a / --author, -k / --keyword]");
}

fn present(e: &PackageEntry, global_options: &GlobalOptions) -> String {
    let verbosity  = global_options.verbose.unwrap_or(2);
    let use_colors = global_options.color != Color::Never;

    let latest_version = e.releases.iter()
        .max_by(|a, b| version_ord(&a.version).cmp(&version_ord(&b.version)))
        .map(|r| r.version.as_str())
        .unwrap_or("?");

    let name_ver = if use_colors {
        Green.paint(format!("{} {}", e.package.name, latest_version)).to_string()
    } else {
        format!("{} {}", e.package.name, latest_version)
    };

    match verbosity {
        1 => name_ver,
        2 => format!("{} by {}", name_ver, e.package.author),
        _ => {
            let desc = e.package.description.as_deref()
                .unwrap_or("");
            format!("{} by {} - {}", name_ver, e.package.author, desc)
        }
    }
}

/// Parse a version string into a comparable tuple, best-effort.
/// "16" → (16,0,0), "2.1.0" → (2,1,0), unparseable → (0,0,0)
fn version_ord(v: &str) -> (u64, u64, u64) {
    let parts: Vec<u64> = v.split('.')
        .map(|p| p.parse().unwrap_or(0))
        .collect();
    (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    )
}

pub fn system_to_dir(system: &InteractiveFictionSystem) -> Option<&'static str> {
    match system {
        InteractiveFictionSystem::Tads3                                        => Some("tads3"),
        InteractiveFictionSystem::Inform   => Some("inform"),
        InteractiveFictionSystem::Inform6  => Some("inform6"),
        InteractiveFictionSystem::Dialog                                        => Some("dialog"),
        _                                                                        => None,
    }
}
