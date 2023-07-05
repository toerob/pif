extern crate ansi_term;
extern crate clap;
extern crate serde;
extern crate git2;
extern crate sublime_fuzzy;
extern crate glob;
extern crate globwalk;
extern crate dotenv;
extern crate home;
extern crate dirs;
extern crate semver;
extern crate online;
extern crate mockall;

mod detect;
mod args;
mod model;
mod update;
mod info;
mod install;
mod list;
mod makefile;
mod color;
mod common;


use std::{fs::{self}, process::exit};

use args::{InteractiveFictionToolArgs, MenuSubCommand};
use clap::{Parser};
use info::extensions_info;
use install::install_extensions;
use list::list_extensions;
use semver::VersionReq;
use update::update_extensions;

use mockall::*;
use mockall::predicate::*;



// TODO: an info option that scans the folder and displays installed extensions along with their descriptions?
// TODO: installation recipe that creates .ifp folder in the home directory. Clone settings to this
fn main() -> () {

    let home_dir = dirs::home_dir().expect("Could not determine home directory. ");
    //let ifp_home_dir = home_dir.join(".ifp");
    let ifp_settings_dir = home_dir.join(".ifp/settings");
    fs::create_dir_all(&ifp_settings_dir).expect("Could not create home and settings directory for ifp.");
    let workspace_folder = ifp_settings_dir.as_os_str().to_str().expect("Conversion from PathBuf to str failed");


    let choice = InteractiveFictionToolArgs::parse();
    match choice.menu {
        MenuSubCommand::Update(_) => update_extensions(&choice.global_options, &workspace_folder),
        MenuSubCommand::Info(cmd_args) => extensions_info(&cmd_args.name, &choice.global_options),
        MenuSubCommand::List(cmd_args) => list_extensions(&cmd_args.list_options, &choice.global_options),
        MenuSubCommand::Install(cmd_args) => install_extensions(&cmd_args.name, &choice.global_options),
    }
}




#[automock]
trait Hello {
    fn kalle(&self) -> u32  {
        return 34;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mockall() {
        println!("###");
        let mut mockHello = MockHello::new();
        
        mockHello.expect_kalle()
            .returning(|| 112);

        let x = mockHello.kalle();
        println!("**{}", &x.to_string());
        assert_eq!(x, 112);


    }
}


/*



    //dotenv().expect("DotEnv could not load");
    //let test_env = std::env::var("HOME").expect("HOME environment variable must be set.");


    let mut cmd = Command::new("repl")
        .version("1.0.0")
        .propagate_version(true)
        .multicall(true)
        .subcommand(Command::new("foo").subcommand(Command::new("bar").arg(Arg::new("value"))));
    cmd.build();*/
