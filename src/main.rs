extern crate clap;
extern crate serde;

mod args;
mod model;

use args::{InteractiveFictionToolArgs, ListOptions, MenuSubCommand};
use clap::Parser;
use model::Extensions;
use std::{error::Error, fs};

fn main() -> () {
    let args = InteractiveFictionToolArgs::parse();
    println!("{:?}", args);

    match &args.menu {
        MenuSubCommand::Install(cmd_args) => {
            install_extension(&cmd_args.name);
        }
        MenuSubCommand::List(cmd_args) => {
            //println!("{:?}", cmd_args)}
            list_extensions(&cmd_args.list_options)
        }
    }

    //if args.main.has_some()
    //args.main

    //list_extensions();
    //install_extension();
}

fn install_extension(name: &String) -> Result<(), Box<dyn Error>> {
    let extension_data_str = fs::read_to_string("./extensions.json").unwrap();
    //println!("raw string = {:?}", extensionDataStr);
    let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();

    let filtered: Vec<_> = data
        .extensions
        .iter()
        .filter(|e| e.name.eq_ignore_ascii_case(&name))
        .collect();

    println!("data = {}", filtered[0].name);

    //TODO: fetch
    //let resp = reqwest::blocking::get("https://httpbin.org/ip")?.text()?;
    //println!("{:#?}", resp);

    Ok(())
}

#[warn(unused_attributes)]
fn list_extensions(listOptions: &ListOptions) -> () {
    let hasAuthor = match listOptions.author {
        Some(_) => true,
        _ => false,
    };
    let hasKeyword = match listOptions.keyword {
        Some(_) => true,
        _ => false,
    };

    let extension_data_str = fs::read_to_string("./extensions.json").unwrap();
    //println!("raw string = {:?}", extension_data_str);
    let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();

    //let filtered: Vec<Extension> = data.extensions.iter().filter_map(|f| *f.name == "").collect();

    //let finally_working_save_this = &listOptions.author.to_owned().unwrap_or_else(||"...".to_string());

    //let x = listOptions.author. or_else("");

    if (hasAuthor) {
        let author: &String = listOptions.author.as_ref().unwrap();
        let filtered: Vec<_> = data
            .extensions
            .iter()
            .filter(|e| e.author.as_ref().unwrap().eq_ignore_ascii_case(author))
            .collect();

        for ele in filtered {
            println!("data = {:?}", ele.name);
        }
    }
}
