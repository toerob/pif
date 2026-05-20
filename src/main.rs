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

use args::{ConfigAction, InteractiveFictionToolArgs, MenuSubCommand, RegistryAction};
use settings::{load_config, reset_all_install_dirs, reset_install_dir, set_install_dir};
use clap::Parser;

use db::{clean_stale_installations, get_or_create_table, print_installations, remove_installation};
use info::extensions_info;
use install::install_extensions;
use list::{list_extensions, list_tags, search_extensions};
use publish::publish_extension;
use update::update_extensions;

fn main() {
    let conn = get_or_create_table().expect("Could not open registry database");
    let _ = clean_stale_installations(&conn);

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
        MenuSubCommand::Search(cmd_args) => {
            search_extensions(&cmd_args.query, &cmd_args.list_options, &choice.global_options, update_needed)
        }
        MenuSubCommand::Config(cmd_args) => {
            match cmd_args.action {
                ConfigAction::SetDir(args) => {
                    match set_install_dir(&args.sys, &args.directory) {
                        Ok(path) => println!("Updated install dir for '{}' to '{}'\n  Config: {}", args.sys, args.directory, path.display()),
                        Err(e)   => eprintln!("Could not update config: {}", e),
                    }
                }
                ConfigAction::ResetDir(args) => {
                    match args.sys {
                        Some(ref sys) => match reset_install_dir(sys) {
                            Ok((path, true))  => println!("Reset install dir for '{}' to default.\n  Config: {}", sys, path.display()),
                            Ok((_, false))    => println!("No override was set for '{}'.", sys),
                            Err(e)            => eprintln!("Could not update config: {}", e),
                        },
                        None => match reset_all_install_dirs() {
                            Ok((path, 0)) => println!("No install dir overrides were set.\n  Config: {}", path.display()),
                            Ok((path, n)) => println!("Reset {} install dir override{}.\n  Config: {}", n, if n == 1 { "" } else { "s" }, path.display()),
                            Err(e)        => eprintln!("Could not update config: {}", e),
                        },
                    }
                }
                ConfigAction::ListDir => {
                    let dirs = load_config().install_dirs;
                    if dirs.is_empty() {
                        println!("No install directories configured.");
                    } else {
                        let use_color = choice.global_options.color != args::Color::Never;
                        for (system, dir) in &dirs {
                            if use_color {
                                println!("{}: {}", system, ansi_term::Colour::Yellow.paint(dir));
                            } else {
                                println!("{}: {}", system, dir);
                            }
                        }
                    }
                }
            }
        }
        MenuSubCommand::Tags(_) => list_tags(&choice.global_options, update_needed),
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
