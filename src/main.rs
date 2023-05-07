extern crate ansi_term;
extern crate clap;
extern crate serde;
extern crate git2;

mod args;
mod install;
mod list;
mod model;
//mod download;

use args::{InteractiveFictionToolArgs, MenuSubCommand};
use clap::Parser;
use install::install_extensions;
use list::list_extensions;

fn main() -> () {
    match InteractiveFictionToolArgs::parse().menu {
        MenuSubCommand::List(cmd_args) => list_extensions(&cmd_args.list_options),
        MenuSubCommand::Install(cmd_args) => install_extensions(&cmd_args.name),
    }
}
