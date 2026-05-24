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
pub mod config;
mod gitops;
mod db;

use std::fs;
use args::{
    ConfigAction, DirAction, InteractiveFictionToolArgs, MenuSubCommand,
    RegistryAction, SystemsAction, VerboseAction, VersionsAction,
};
use config::{
    get_main_config_file,
    reset_all_install_dirs, reset_install_dir,
    reset_verbose_level, set_verbose_level,
    set_install_dir,
    set_systems, add_systems, remove_systems, reset_systems,
    set_system_versions, add_system_version_specs, remove_system_version_specs, reset_system_versions,
};
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
            extensions_info(&cmd_args, &choice.global_options, update_needed)
        }
        MenuSubCommand::List(cmd_args) => {
            list_extensions(&cmd_args.list_options, &choice.global_options, update_needed)
        }
        MenuSubCommand::Install(cmd_args) => {
            install_extensions(
                &cmd_args.names,
                &cmd_args.install_options,
                &choice.global_options,
                update_needed,
            )
        }
        MenuSubCommand::Publish(cmd_args) => {
            publish_extension(&cmd_args.directory, &choice.global_options)
        }
        MenuSubCommand::Search(cmd_args) => {
            search_extensions(&cmd_args.query, &cmd_args.list_options, &choice.global_options, update_needed)
        }
        MenuSubCommand::Config(cmd_args) => {
            let use_color = choice.global_options.color != args::Color::Never;
            match cmd_args.action {
                ConfigAction::Dir(dir_cmd) => match dir_cmd.action {
                    DirAction::Set(a) => match set_install_dir(&a.sys, &a.directory) {
                        Ok(_)  => show_config(use_color, &[a.directory.as_str()]),
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                    DirAction::Reset(a) => match a.sys {
                        Some(ref sys) => match reset_install_dir(sys) {
                            Ok((path, true))  => println!("Install dir for '{}' reset to default.\n  Config: {}", sys, path.display()),
                            Ok((_, false))    => println!("No override was set for '{}'.", sys),
                            Err(e)            => eprintln!("Could not update config: {}", e),
                        },
                        None => match reset_all_install_dirs() {
                            Ok((path, 0)) => println!("No install dir overrides were set.\n  Config: {}", path.display()),
                            Ok((path, n)) => println!("Reset {} install dir override{}.\n  Config: {}", n, if n == 1 { "" } else { "s" }, path.display()),
                            Err(e)        => eprintln!("Could not update config: {}", e),
                        },
                    },
                },
                ConfigAction::Verbose(verb_cmd) => match verb_cmd.action {
                    VerboseAction::Set(a) => {
                        let level_str = a.level.to_string();
                        match set_verbose_level(a.level) {
                            Ok(_)  => show_config(use_color, &[level_str.as_str()]),
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                    VerboseAction::Reset => match reset_verbose_level() {
                        Ok((path, true))  => println!("Verbose level reset to default (2).\n  Config: {}", path.display()),
                        Ok((_, false))    => println!("No verbose level override was set."),
                        Err(e)            => eprintln!("Could not update config: {}", e),
                    },
                },
                ConfigAction::Systems(sys_cmd) => match sys_cmd.action {
                    SystemsAction::Set(a) => match set_systems(&a.systems) {
                        Ok(_) => {
                            let refs: Vec<&str> = a.systems.iter().map(|s| s.as_str()).collect();
                            show_config(use_color, &refs);
                        }
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                    SystemsAction::Add(a) => match add_systems(&a.systems) {
                        Ok(_) => {
                            let refs: Vec<&str> = a.systems.iter().map(|s| s.as_str()).collect();
                            show_config(use_color, &refs);
                        }
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                    SystemsAction::Remove(a) => match remove_systems(&a.systems) {
                        Ok((_, removed)) if removed.is_empty() => {
                            println!("No matching systems found to remove.");
                        }
                        Ok(_) => show_config(use_color, &[]),
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                    SystemsAction::Reset => match reset_systems() {
                        Ok((path, true))  => println!("Systems filter removed.\n  Config: {}", path.display()),
                        Ok((_, false))    => println!("No systems filter was set."),
                        Err(e)            => eprintln!("Could not update config: {}", e),
                    },
                },
                ConfigAction::Versions(ver_cmd) => match ver_cmd.action {
                    VersionsAction::Set(a) => match set_system_versions(&a.sys, &a.versions) {
                        Ok(_) => {
                            let refs: Vec<&str> = a.versions.iter().map(|s| s.as_str()).collect();
                            show_config(use_color, &refs);
                        }
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                    VersionsAction::Add(a) => match add_system_version_specs(&a.sys, &a.versions) {
                        Ok(_) => {
                            let refs: Vec<&str> = a.versions.iter().map(|s| s.as_str()).collect();
                            show_config(use_color, &refs);
                        }
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                    VersionsAction::Remove(a) => match remove_system_version_specs(&a.sys, &a.versions) {
                        Ok((_, removed)) if removed.is_empty() => {
                            println!("No matching version specs found to remove.");
                        }
                        Ok(_) => show_config(use_color, &[]),
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                    VersionsAction::Reset(a) => match reset_system_versions(a.sys.as_deref()) {
                        Ok((path, true)) => match a.sys {
                            Some(ref sys) => println!("Version specs for '{}' removed.\n  Config: {}", sys, path.display()),
                            None          => println!("All version specs removed.\n  Config: {}", path.display()),
                        },
                        Ok((_, false)) => match a.sys {
                            Some(ref sys) => println!("No version specs were set for '{}'.", sys),
                            None          => println!("No version specs were set."),
                        },
                        Err(e) => eprintln!("Could not update config: {}", e),
                    },
                },
                ConfigAction::Show => show_config(use_color, &[]),
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

fn show_config(use_color: bool, highlight: &[&str]) {
    match get_main_config_file() {
        Ok(path) => {
            if use_color {
                println!("{}", ansi_term::Colour::Yellow.paint(format!("Config: {}", path.display())));
            } else {
                println!("Config: {}", path.display());
            }
            println!();
            match fs::read_to_string(&path) {
                Ok(content) => color::print_yaml_colored(&content, use_color, highlight),
                Err(e)      => eprintln!("Could not read config: {}", e),
            }
        }
        Err(e) => eprintln!("Could not locate config file: {}", e),
    }
}
