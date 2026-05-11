extern crate ansi_term;
extern crate clap;
extern crate serde;

use crate::args;
use crate::{
    args::{ Color, GlobalOptions, InteractiveFictionSystem, SortProperty },
    detect::{ detect_system, get_extension_path },
    update::{ update_extensions },
};
use ansi_term::Colour::*;
use crate::model;

use std::fs;
use sublime_fuzzy::FuzzySearch;

//use model::Extensions;

#[warn(unused_attributes)]
pub fn list_extensions(
    list_options: &args::ListOptions,
    global_options: &args::GlobalOptions,
    update_needed: bool
) -> () {

    if update_needed {
        update_extensions(global_options);
    }

    let system_type = if global_options.system == InteractiveFictionSystem::Auto {
        detect_system().0
    } else {
        global_options.system.clone()
    };

    println!("{}", Yellow.paint(format!("System: {:?}", system_type)).to_string());
    let systems: Vec<InteractiveFictionSystem> = if system_type == InteractiveFictionSystem::Unknown {
        vec![InteractiveFictionSystem::Tads3, InteractiveFictionSystem::Dialog, InteractiveFictionSystem::Inform6]
    } else {
        vec![system_type]
    };

    for system_type in systems {
        list_for_system(system_type, list_options, global_options);
    }
}

fn list_for_system(
    system_type: InteractiveFictionSystem,
    list_options: &args::ListOptions,
    global_options: &args::GlobalOptions,
) {
    let file_path = match get_extension_path(system_type.clone()) {
        Some(p) => p,
        None => return,
    };

    // TODO: use repo_dir to get the latest json configuration file
    let config_file = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo")
        .join(&file_path)
        .clone();

    // println!("file_path: {} ", &config_file.clone().display());

    //OLD: let extension_data_str = fs::read_to_string(file_path).unwrap();
    let extension_data_str = match fs::read_to_string(&config_file) {
        Ok(s) => s,
        Err(_) => return,
    };

    let data: model::Extensions = serde_yaml::from_str(&extension_data_str).unwrap();

    for warning in data.validate() {
        eprintln!("Schema warning: {}", warning);
    }

    let mut extensions = data.extensions;

    if list_options.author.is_some() {
        let author = list_options.author.as_ref().unwrap().to_owned().to_lowercase();

        extensions = extensions
            .into_iter()
            .filter(|e| {
                FuzzySearch::new(&author, &e.author.as_ref().unwrap().to_lowercase())
                    .case_insensitive()
                    .best_match()
                    .is_some()
            })
            .collect();
    }
    if list_options.keyword.is_some() {
        let keyword = list_options.keyword.as_ref().unwrap().to_owned().to_lowercase();

        extensions = extensions
            .into_iter()
            .filter(|e| {
                FuzzySearch::new(&keyword, &e.name.to_lowercase())
                    .case_insensitive()
                    .best_match()
                    .is_some()
            })
            .collect();
    }

    // TODO: implement OrderingDirection -> SortOrderDir

    if SortProperty::Name == list_options.sort_property {
        extensions.sort_by_key(|e| e.to_owned().name);
    } else if SortProperty::Author == list_options.sort_property {
        extensions.sort_by_key(|e| e.to_owned().author);
    } else if SortProperty::Date == list_options.sort_property {
        // TODO: compare the version's version number (semver wise) to find out the latest version to compare with

        extensions.sort_by_key(|e| {
            match &e.versions.get(0) {
                Some(v) => v.last_modified.to_owned(),
                _ => Some(String::from("0")),
            }
        });
    }

    let delimiter = if list_options.presentation == args::ListPresentation::Comma {
        ","
    } else {
        "\n"
    };

    let na: Vec<_> = extensions
        .iter()
        .map(|e| create_presentation(e, global_options))
        .collect();

    let str = na.join(delimiter);
    println!("{}", str);
    println!("");
    println!("[Filter by -a / --author, -k / --keyword]");
}

fn create_presentation(e: &crate::model::Extension, global_options: &GlobalOptions) -> String {
    let verbosity_level = global_options.verbose.unwrap();
    let use_colors = if Color::Never == global_options.color { false } else { true };

    //for version in e.versions.to_owned() {
    //    print!("{:?}", &version.version);
    //}
    //println!();
    let mut extension_versions = e.versions.to_owned();
    extension_versions.sort_by_key(|v| v.to_owned().version.unwrap_or(semver::Version::new(0, 0, 0)));

    let latest_version = extension_versions.last().unwrap();
    let version = match &latest_version.version {
        Some(v) if v.major == 0 && v.minor == 0 && v.patch == 0 && v.pre.as_str() == "SNAPSHOT" => {
            "SNAPSHOT".to_string()
        }
        Some(v) => v.to_string(),
        None => "LATEST".to_string(),
    };

    let name = if use_colors {
        Green.paint(format!("{} {} ", e.name.as_str(), &version)).to_string()
    } else {
        e.name.as_str().to_owned()
    };

    return match verbosity_level {
        1 => name,
        2 => {
            name +
                " (" +
                latest_version.last_modified.as_ref().unwrap().to_owned().as_str() +
                ")" +
                " by " +
                e.author.as_ref().unwrap().to_owned().as_str()
        }
        _ => {
            name +
                " (" +
                latest_version.last_modified.as_ref().unwrap().to_owned().as_str() +
                ")" +
                " by " +
                e.author.as_ref().unwrap().to_owned().as_str().trim_end() +
                " - " +
                e.desc.as_ref().unwrap().to_owned().as_str()
        }
    };
}

/*
//let filteredData = filter_by_author(listOptions.author.as_ref().unwrap().to_owned(), &extensions);

fn filter_by_author(author: String, data: &Vec<Extension>) -> Vec<&Extension> {
    println!("Filter by author *{}*", author);
    return data
        .iter()
        .filter(|e| e.author.as_ref().unwrap().eq_ignore_ascii_case(&author))
        .collect();
}
 */

//let filtered: Vec<Extension> = data.extensions.iter().filter_map(|f| *f.name == "").collect();
//let finally_working_save_this = &listOptions.author.to_owned().unwrap_or_else(||"...".to_string());
//let x = listOptions.author. or_else("");
