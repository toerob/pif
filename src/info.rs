use ansi_term::Colour::*;

use crate::{
    args::{Color, GlobalOptions, InteractiveFictionSystem},
    color::{create_info_message, create_success_msg},
    detect::detect_system,
    list::system_to_dir,
    model::{load_registry, PackageEntry},
    update::{get_registry_root, update_extensions},
};

pub fn extensions_info(
    names: &[String],
    global_options: &GlobalOptions,
    update_needed: bool,
) {
    if update_needed {
        update_extensions(global_options);
    }

    if names.is_empty() {
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

    // Info always searches all systems — no filter.
    let entries = match load_registry(&registry_root, None) {
        Ok(e) => e,
        Err(e) => { eprintln!("Could not load registry: {}", e); return; }
    };

    let lowercase_names: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();

    let matches: Vec<&PackageEntry> = entries.iter().filter(|e| {
        let id   = e.package.id.to_lowercase();
        let name = e.package.name.to_lowercase();
        lowercase_names.iter().any(|q| id.starts_with(q) || name.starts_with(q))
    }).collect();

    if matches.is_empty() {
        println!("{}", Red.paint(format!("No extension found for: {}", names.join(", "))));
        return;
    }

    if detected_system != InteractiveFictionSystem::Unknown {
        println!("{}\n", Yellow.paint(format!("[Detected system: {:?}]", detected_system)));
    }

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

        println!("Available versions:");
        let mut releases = entry.releases.clone();
        releases.sort_by(|a, b| version_ord(&a.version).cmp(&version_ord(&b.version)));
        let total = releases.len();

        for (i, loaded) in releases.iter().enumerate() {
            let ver_str = create_info_message(use_colors, loaded.version.clone());
            let url = loaded.release.source.as_ref()
                .map(|s| s.url.as_str())
                .unwrap_or("(no url)");
            let date = loaded.release.date.as_deref().unwrap_or("-");
            let latest = if i == total - 1 {
                Green.paint(" <== LATEST").to_string()
            } else {
                String::new()
            };
            println!("  * {} {}  ({}){}", ver_str, url, date, latest);
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
