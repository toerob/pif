use crate::{
    args::{self, Color, InteractiveFictionSystem},
    color::{create_info_message, create_success_msg},
};
use ansi_term::Colour::*;
use semver;
use std::fs::{self};
use dirs_next;

use crate::model::Extensions;
use crate::{
    detect::{detect_system, get_extension_path},
    update::update_extensions,
};

pub fn extensions_info(
    names: &[String],
    global_options: &args::GlobalOptions,
    update_needed: bool,
) {
    if update_needed {
        update_extensions(global_options);
    }

    let _verbosity_level = global_options.verbose.unwrap();
    let use_colors = if Color::Never == global_options.color {
        false
    } else {
        true
    };

    if names.len() == 0 {
        println!(
        "{}",Red.paint(format!("No packages specified. Command usage examples: \n  \"pif install abc \"\n  \"pif install abc def\""))
    );
        return;
    }

    let system_type = if global_options.system == InteractiveFictionSystem::Auto {
        detect_system().0
    } else {
        global_options.system.clone()
    };

    let systems: Vec<InteractiveFictionSystem> = if system_type == InteractiveFictionSystem::Unknown
    {
        vec![
            InteractiveFictionSystem::Tads3,
            InteractiveFictionSystem::Dialog,
            InteractiveFictionSystem::Inform6,
        ]
    } else {
        vec![system_type]
    };

    for system in systems {
        let file_path = match get_extension_path(system.clone()) {
            Some(p) => p,
            None => continue,
        };

        println!("{}\n", Yellow.paint(format!("[System: {:?}]", system)));

        let config_file = dirs_next::data_dir()
            .expect("Could not determine data directory")
            .join("pif")
            .join("repo")
            .join(&file_path);

        let extension_data_str = match fs::read_to_string(&config_file) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let data: Extensions = serde_yaml::from_str(&extension_data_str).unwrap();

        for warning in data.validate() {
            eprintln!("Schema warning: {}", warning);
        }

        let lowercase_names: Vec<String> = names
            .to_owned()
            .into_iter()
            .map(|it| it.to_lowercase())
            .collect();

        let extensions_info = data.extensions.iter().filter(|ext| {
            let extension_name = ext.to_owned().name.to_lowercase();
            for lowercase_name in lowercase_names.to_owned().into_iter() {
                if extension_name.to_owned().starts_with(&lowercase_name) {
                    return true;
                }
            }
            return false;
        });

        for (_, ele) in extensions_info.enumerate() {
            let name = match use_colors {
                true => create_success_msg(use_colors, ele.name.clone()),
                _ => ele.name.clone(),
            };

            let author = ele.author.as_deref().unwrap_or("unknown");
            let desc = ele.desc.as_deref().unwrap_or("");
            println!("{} by {} \n{}\n", name, author, desc);

            if ele.versions.to_owned().len() > 0 {
                println!("Available versions: ");
                let mut sorted_versions = ele.clone();

                sorted_versions.versions.sort_by_key(|e| {
                    e.to_owned()
                        .version
                        .unwrap_or(semver::Version::new(0, 0, 0))
                });

                let total = sorted_versions.versions.len() - 1;

                for (idx, version) in sorted_versions.versions.into_iter().enumerate() {
                    let v_str = version
                        .version
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let version_number = create_info_message(use_colors, v_str);
                    let url = version.url.as_ref().unwrap();
                    let last_modified = version.last_modified.as_ref().unwrap();
                    let latest = if idx == total {
                        Green.paint("<== LATEST").to_string()
                    } else {
                        String::from("")
                    };
                    println!(
                        "  *  {} {}  ({}) {} ",
                        version_number, url, last_modified, latest
                    );
                }
                println!();
            }
        }
    }
}
