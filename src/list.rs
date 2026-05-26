use ansi_term::Colour::*;
use sublime_fuzzy::FuzzySearch;

use crate::{
    args::{Color, GlobalOptions, InteractiveFictionSystem, ListOptions, ListPresentation, SortProperty},
    detect::detect_system,
    model::{load_registry, PackageEntry},
    config::{load_config, version_matches_any, VersionSpec},
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

    let config = load_config();

    let system_type = if global_options.system == InteractiveFictionSystem::Auto {
        detect_system().0
    } else {
        global_options.system.clone()
    };

    // Config systems filter applies in Auto mode when no project is detected
    let apply_config_systems = global_options.system == InteractiveFictionSystem::Auto
        && system_type == InteractiveFictionSystem::Unknown
        && !config.systems.is_empty();

    if apply_config_systems {
        println!("{}", Yellow.paint(format!("Systems: [{}]", config.systems.join(", "))));
    } else if system_type != InteractiveFictionSystem::Unknown {
        println!("{}", Yellow.paint(format!("System: {:?}", system_type)));
    }

    let registry_root = get_registry_root();

    let system_filter = if apply_config_systems { None } else { system_to_dir(&system_type) };

    let mut entries = match load_registry(&registry_root, system_filter) {
        Ok(e) => e,
        Err(e) => { eprintln!("Could not load registry: {}", e); return; }
    };

    if apply_config_systems {
        entries.retain(|e| config.systems.iter().any(|s| s == &e.system));
    }

    apply_version_filter(&mut entries, &config.system_versions);

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
    if let Some(tag) = &list_options.tag {
        let q = tag.to_lowercase();
        entries.retain(|e| {
            e.package.tags.as_deref().unwrap_or(&[])
                .iter().any(|t| t.to_lowercase() == q)
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
    println!("[Filter by -a / --author, -k / --keyword, -t / --tag]");
    if apply_config_systems {
        println!("[Showing [{}] only — use --system all to see all systems]", config.systems.join(", "));
    } else if system_filter.is_some() {
        println!("[Showing {:?} extensions only — use --system all to see all systems]", system_type);
    }
}

fn present(e: &PackageEntry, global_options: &GlobalOptions) -> String {
    let verbosity  = global_options.verbose;
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

pub fn search_extensions(
    query: &str,
    list_options: &ListOptions,
    global_options: &GlobalOptions,
    update_needed: bool,
) {
    if update_needed {
        update_extensions(global_options);
    }

    let config = load_config();

    // Search ignores the auto-detected system so results span all systems,
    // but the config systems filter still applies when --system is not given.
    let system_type = if global_options.system == InteractiveFictionSystem::Auto {
        InteractiveFictionSystem::All
    } else {
        global_options.system.clone()
    };

    let apply_config_systems = global_options.system == InteractiveFictionSystem::Auto
        && !config.systems.is_empty();

    if system_type != InteractiveFictionSystem::Unknown
        && system_type != InteractiveFictionSystem::All
    {
        println!("{}", Yellow.paint(format!("System: {:?}", system_type)));
    }

    let registry_root = get_registry_root();
    let system_filter = if apply_config_systems { None } else { system_to_dir(&system_type) };

    let mut entries = match load_registry(&registry_root, system_filter) {
        Ok(e) => e,
        Err(e) => { eprintln!("Could not load registry: {}", e); return; }
    };

    if apply_config_systems {
        entries.retain(|e| config.systems.iter().any(|s| s == &e.system));
    }

    apply_version_filter(&mut entries, &config.system_versions);

    let q = query.to_lowercase();
    entries.retain(|e| {
        let name_match = e.package.name.to_lowercase().contains(&q);
        let desc_match = e.package.description.as_deref()
            .map(|d| d.to_lowercase().contains(&q))
            .unwrap_or(false);
        name_match || desc_match
    });

    if let Some(author) = &list_options.author {
        let aq = author.to_lowercase();
        entries.retain(|e| {
            FuzzySearch::new(&aq, &e.package.author.to_lowercase())
                .case_insensitive().best_match().is_some()
        });
    }
    if let Some(keyword) = &list_options.keyword {
        let kq = keyword.to_lowercase();
        entries.retain(|e| {
            FuzzySearch::new(&kq, &e.package.name.to_lowercase())
                .case_insensitive().best_match().is_some()
        });
    }
    if let Some(tag) = &list_options.tag {
        let tq = tag.to_lowercase();
        entries.retain(|e| {
            e.package.tags.as_deref().unwrap_or(&[])
                .iter().any(|t| t.to_lowercase() == tq)
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

    if entries.is_empty() {
        println!("No extensions found matching '{}'.", query);
        return;
    }

    let delimiter = if list_options.presentation == ListPresentation::Comma { "," } else { "\n" };
    let lines: Vec<_> = entries.iter().map(|e| present(e, global_options)).collect();
    println!("{}", lines.join(delimiter));
    println!();
    println!("[Filter by -a / --author, -k / --keyword, -t / --tag]");
}

pub fn list_tags(global_options: &GlobalOptions, update_needed: bool) {
    if update_needed {
        update_extensions(global_options);
    }

    let config = load_config();

    let system_type = if global_options.system == InteractiveFictionSystem::Auto {
        detect_system().0
    } else {
        global_options.system.clone()
    };

    let apply_config_systems = global_options.system == InteractiveFictionSystem::Auto
        && system_type == InteractiveFictionSystem::Unknown
        && !config.systems.is_empty();

    let registry_root = get_registry_root();
    let system_filter = if apply_config_systems { None } else { system_to_dir(&system_type) };

    let mut entries = match load_registry(&registry_root, system_filter) {
        Ok(e) => e,
        Err(e) => { eprintln!("Could not load registry: {}", e); return; }
    };

    if apply_config_systems {
        entries.retain(|e| config.systems.iter().any(|s| s == &e.system));
    }

    apply_version_filter(&mut entries, &config.system_versions);

    let mut tags: Vec<String> = entries.iter()
        .flat_map(|e| e.package.tags.as_deref().unwrap_or(&[]).iter().cloned())
        .collect();
    tags.sort();
    tags.dedup();

    if tags.is_empty() {
        println!("No tags found.");
        return;
    }

    for tag in &tags {
        println!("{}", tag);
    }
}

fn apply_version_filter(
    entries: &mut Vec<PackageEntry>,
    system_versions: &std::collections::HashMap<String, Vec<String>>,
) {
    for entry in entries.iter_mut() {
        if let Some(specs_raw) = system_versions.get(&entry.system) {
            let specs: Vec<VersionSpec> = specs_raw.iter().map(|s| VersionSpec::parse(s)).collect();
            entry.releases.retain(|r| version_matches_any(&r.version, &specs));
        }
    }
    entries.retain(|e| !e.releases.is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::model::{LoadedRelease, Package, PackageEntry, Release};

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

    fn make_entry(system: &str, versions: &[&str]) -> PackageEntry {
        PackageEntry {
            system: system.to_string(),
            namespace: "test".to_string(),
            package: Package {
                schema_version: 1,
                id: "test-pkg".to_string(),
                name: "test-pkg".to_string(),
                author: "Author".to_string(),
                description: None,
                tags: None,
            },
            releases: versions.iter().map(|v| make_release(v)).collect(),
        }
    }

    // ── no config → passthrough ──────────────────────────────────────────────

    #[test]
    fn no_config_passes_all_releases() {
        let mut entries = vec![make_entry("inform", &["16-i10.1", "16-i11.0"])];
        apply_version_filter(&mut entries, &HashMap::new());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].releases.len(), 2);
    }

    // ── matching releases kept, non-matching dropped ─────────────────────────

    #[test]
    fn keeps_only_matching_releases() {
        let mut entries = vec![make_entry("inform", &["16-i10.1", "17-i10.2", "16-i11.0"])];
        let config = HashMap::from([("inform".into(), vec!["i10".into()])]);
        apply_version_filter(&mut entries, &config);
        assert_eq!(entries[0].releases.len(), 2);
        assert!(entries[0].releases.iter().all(|r| r.version.contains("i10")));
    }

    #[test]
    fn multiple_specs_or_combined() {
        let mut entries = vec![make_entry("inform", &["16-i10.1", "16-i11.0", "16-i12.0"])];
        let config = HashMap::from([("inform".into(), vec!["i10".into(), "i11".into()])]);
        apply_version_filter(&mut entries, &config);
        let versions: Vec<&str> = entries[0].releases.iter().map(|r| r.version.as_str()).collect();
        assert_eq!(versions, ["16-i10.1", "16-i11.0"]);
    }

    #[test]
    fn plain_version_prefix_match() {
        let mut entries = vec![make_entry("tads3", &["3.1.0", "3.1.2", "3.2.0"])];
        let config = HashMap::from([("tads3".into(), vec!["3.1".into()])]);
        apply_version_filter(&mut entries, &config);
        assert_eq!(entries[0].releases.len(), 2);
        assert!(entries[0].releases.iter().all(|r| r.version.starts_with("3.1")));
    }

    // ── entry removed when no releases survive ───────────────────────────────

    #[test]
    fn entry_removed_when_no_releases_match() {
        let mut entries = vec![make_entry("inform", &["16-i11.0", "17-i11.1"])];
        let config = HashMap::from([("inform".into(), vec!["i10".into()])]);
        apply_version_filter(&mut entries, &config);
        assert!(entries.is_empty());
    }

    // ── only configured system is affected ───────────────────────────────────

    #[test]
    fn unconfigured_system_passes_through_untouched() {
        let mut entries = vec![
            make_entry("inform", &["16-i10.1"]),
            make_entry("tads3", &["1.0", "2.0"]),
        ];
        let config = HashMap::from([("inform".into(), vec!["i10".into()])]);
        apply_version_filter(&mut entries, &config);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].releases.len(), 2);
    }

    // ── multiple entries, mixed outcomes ─────────────────────────────────────

    #[test]
    fn mixed_entries_some_survive_some_removed() {
        let mut entries = vec![
            make_entry("inform", &["16-i10.1", "16-i11.0"]),
            make_entry("inform", &["16-i11.0", "17-i11.1"]),
        ];
        let config = HashMap::from([("inform".into(), vec!["i10".into()])]);
        apply_version_filter(&mut entries, &config);
        // Second entry has no i10 releases → removed
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].releases.len(), 1);
        assert_eq!(entries[0].releases[0].version, "16-i10.1");
    }
}

pub fn system_to_dir(system: &InteractiveFictionSystem) -> Option<&'static str> {
    match system {
        InteractiveFictionSystem::Tads3   => Some("tads3"),
        InteractiveFictionSystem::Tads2   => Some("tads2"),
        InteractiveFictionSystem::Dialog  => Some("dialog"),
        InteractiveFictionSystem::Inform  => Some("inform"),
        InteractiveFictionSystem::Inform6 => Some("inform6"),
        InteractiveFictionSystem::Hugo    => Some("hugo"),
        InteractiveFictionSystem::Zil     => Some("zil"),
        _                                 => None,
    }
}
