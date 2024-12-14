use std::{ io::{ stdout, Write }, process::exit };
use std::{ fs::{ self, File }, io::{ Cursor }, path::Path };
use ansi_term::Colour::*;

use online::check;
use crate::{ 
    args::{ GlobalOptions, Color }, 
    color::{ print_success_msg, print_warning_msg },
    settings::get_main_config_file,
    gitops::{get_or_create_repo_dir, clone_or_pull_repo},
};
use git2::{
    Error,
    ErrorCode,
    Repository,
    FetchOptions,
    StatusOptions,
    MergeOptions,
    RemoteCallbacks,
    Remote,
    Cred,
};

use std::path::PathBuf;
use dirs_next::data_dir;

use config::Config;
use std::collections::HashMap;

pub fn update_extensions(global_options: &GlobalOptions) {
    let use_colour = Color::Never != global_options.color;
    let success_color = match Color::Never != global_options.color {
        true => Some(Green),
        false => None,
    };

    let config_file_pathbuf = get_main_config_file().unwrap();
    let config_file_path_str = config_file_pathbuf.to_str().unwrap();
    
    println!("***{}***", config_file_path_str);

    let settings = Config::builder()
        .add_source(config::File::with_name(config_file_path_str))
        .build()
        .unwrap();

    let config = settings.clone().try_deserialize::<HashMap<String, String>>().unwrap();

    let repository_url = &settings.get_string("main_repository_url").unwrap();
    let repository_main_branch = &settings.get_string("main_repository_branch").unwrap();

    // println!("{:?}", &repository_url);
    // println!("{:?}", &repository_main_branch);

    let repo_dir = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("ifp")
        .join("repo")
        .clone();

    let repo_dir_str = repo_dir.as_path().to_str().clone().unwrap();

    match get_or_create_repo_dir(&repo_dir_str) {
        Ok(repo_path) => {
            if let Err(e) = clone_or_pull_repo(repository_url, repository_main_branch, &repo_path) {
                print_warning_msg(use_colour, format!("Error: {}\n", e));
            }
        }
        Err(e) => { eprintln!("Failed to create repository directory: {}", e) }
    }
}
