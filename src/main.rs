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
mod publish;
mod update;
pub mod settings;
mod gitops;
mod db;

use args::{InteractiveFictionToolArgs, MenuSubCommand, RegistryAction};
use clap::Parser;

use db::{clean_stale_installations, get_or_create_table, print_installations, remove_installation};
use info::extensions_info;
use install::install_extensions;
use list::list_extensions;
use publish::publish_extension;
use update::update_extensions;

// TODO: make ifarchive possible without maintaining a specific list

fn main() {
    let conn = get_or_create_table().expect("Could not open registry database");

    let repo_dir = dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo");

    let update_needed = !repo_dir.exists();

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
                &cmd_args.names,
                &cmd_args.install_options,
                &choice.global_options,
                update_needed
            )
        }
        MenuSubCommand::Publish(cmd_args) => {
            publish_extension(&cmd_args.directory, &choice.global_options)
        }
        MenuSubCommand::Registry(cmd_args) => {
            match cmd_args.action {
                RegistryAction::List => {
                    if let Err(e) = print_installations(&conn) {
                        eprintln!("Registry error: {}", e);
                    }
                }
                RegistryAction::Remove(args) => {
                    match remove_installation(&conn, &args.name, args.path.as_deref()) {
                        Ok(0) => println!("No matching registry entry found for '{}'.", args.name),
                        Ok(n) => println!("Removed {} registry entr{}.", n, if n == 1 { "y" } else { "ies" }),
                        Err(e) => eprintln!("Registry error: {}", e),
                    }
                }
                RegistryAction::Clean => {
                    match clean_stale_installations(&conn) {
                        Ok(0) => println!("All registry entries are valid."),
                        Ok(n) => println!("Removed {} stale entr{}.", n, if n == 1 { "y" } else { "ies" }),
                        Err(e) => eprintln!("Registry error: {}", e),
                    }
                }
            }
        }
    }
}
