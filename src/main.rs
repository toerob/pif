extern crate ansi_term;
extern crate clap;
extern crate serde;
extern crate git2;
extern crate sublime_fuzzy;
extern crate glob;
extern crate globwalk;

mod detect;
mod args;
mod model;
mod update;
mod install;
mod list;
mod makefile;

use std::{process::exit, fs::DirEntry};

use args::{InteractiveFictionToolArgs, MenuSubCommand};
use clap::{Parser, Command, Arg};
use detect::{detect_system, InteractiveFictionSystem};
use install::install_extensions;
use list::list_extensions;
use update::update_extensions;

// TODO: An update command that pulls from a git repo on "rift/ifp update" and updates extensions.json with new entries, 
// TODO: stores it locally, in a workplace folder or close by within a .folder?

fn main() -> () {
    
    let choice = InteractiveFictionToolArgs::parse();
    match choice.menu {
        MenuSubCommand::Update(_) => update_extensions(&choice.global_options),
        MenuSubCommand::List(cmd_args) => list_extensions(&cmd_args.list_options, &choice.global_options),
        MenuSubCommand::Install(cmd_args) => install_extensions(&cmd_args.name, &choice.global_options),
    }
}



/*
    let mut cmd = Command::new("repl")
        .version("1.0.0")
        .propagate_version(true)
        .multicall(true)
        .subcommand(Command::new("foo").subcommand(Command::new("bar").arg(Arg::new("value"))));
    cmd.build();*/
