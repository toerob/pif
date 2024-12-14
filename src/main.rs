extern crate ansi_term;
extern crate clap;
extern crate dirs;
extern crate dotenv;
extern crate git2;
extern crate glob;
extern crate globwalk;
extern crate home;
extern crate mockall;
extern crate online;
extern crate semver;
extern crate serde;
extern crate sublime_fuzzy;
extern crate regex;
extern crate lazy_static;

mod args;
mod color;
mod common;
mod detect;
mod info;
mod install;
mod list;
mod makefile;
mod model;
mod update;

use std::fs::{self};


use args::{InteractiveFictionToolArgs, MenuSubCommand};
use clap::Parser;
use info::extensions_info;
use install::install_extensions;
use list::list_extensions;
use update::update_extensions3;

use std::path::PathBuf;


// TODO: make ifarchive possible without maintaining a specific list

fn main() -> () {
    // Bring the repository home inside the workspace_folder
    let workspace_folder2 : PathBuf = get_data_dir().expect("Could not determine workspace folder. ");
    print!("Path: {}\n", workspace_folder2.as_path().display());

    let home_dir = dirs::home_dir().expect("Could not determine home directory. ");
    print!("Path: {}\n", home_dir.display());

    let ifp_settings_dir = home_dir.join(".ifp/settings");

    fs::create_dir_all(&ifp_settings_dir)
        .expect("Could not create home and settings directory for ifp.");

    let workspace_folder = ifp_settings_dir
        .as_os_str()
        .to_str()
        .expect("Conversion from PathBuf to str failed");

    let workspace_folder_str_2 = ifp_settings_dir
        .as_os_str()
        .to_str()
        .expect("Conversion from PathBuf to str failed");


    let choice = InteractiveFictionToolArgs::parse();
    match choice.menu {
        //MenuSubCommand::Update(_) => update_extensions(&choice.global_options, &workspace_folder),
        MenuSubCommand::Update(_) => update_extensions3(&choice.global_options, &workspace_folder_str_2),
        MenuSubCommand::Info(cmd_args) => extensions_info(&cmd_args.name, &choice.global_options),
        MenuSubCommand::List(cmd_args) => {
            list_extensions(&cmd_args.list_options, &choice.global_options)
        }
        MenuSubCommand::Install(cmd_args) => {
            install_extensions(&cmd_args.name, &cmd_args.install_options, &choice.global_options)
        }
    }
}



fn get_data_dir() -> Result<PathBuf, std::io::Error> {
    let repo_dir = dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("ifp");
    if !repo_dir.exists() {
        fs::create_dir_all(&repo_dir)?;
    }
    Ok(repo_dir)
}
