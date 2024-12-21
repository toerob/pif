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
pub mod settings;
mod gitops;
mod db;

use args::{ InteractiveFictionToolArgs, MenuSubCommand };
use clap::Parser;

use db::{check_installations, get_or_create_table};


use std::fs::{ self };
use std::process::exit;

use info::extensions_info;
use install::install_extensions;
use list::list_extensions;
use update::{ update_extensions };



// TODO: make ifarchive possible without maintaining a specific list

fn main() -> () {
    
    let conn = get_or_create_table().unwrap();
    check_installations(&conn);
    //exit(0);
    

    //let config_file_pathbuf = get_main_config_file().expect("Main configuration file could not be found");
    // TODO: check if update is needed for first run

    let repo_dir = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo")
        .clone();

    let update_needed = if repo_dir.exists() { false } else { true };

    let choice = InteractiveFictionToolArgs::parse();

    match choice.menu {
        MenuSubCommand::Update(_) => update_extensions(&choice.global_options),
        MenuSubCommand::Info(cmd_args) => {
            extensions_info(&cmd_args.name, &choice.global_options, update_needed)
        }
        MenuSubCommand::List(cmd_args) => {
            list_extensions(&cmd_args.list_options, &choice.global_options, update_needed)
        }
        MenuSubCommand::Install(cmd_args) => {
            install_extensions(
                &cmd_args.name,
                &cmd_args.install_options,
                &choice.global_options,
                update_needed
            )
        }
    }
}
