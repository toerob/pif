extern crate ansi_term;
extern crate clap;
extern crate serde;
use ansi_term::Colour::*;
use args::{ListOptions, ListPresentation};
use model::Extensions;
use std::fs;

// TODO: modules
// TODO: fuzzy search

#[warn(unused_attributes)]
pub fn list_extensions(list_options: &ListOptions) -> () {
    let extension_data_str = fs::read_to_string("./extensions.json").unwrap();
    let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();
    let mut extensions = data.extensions;

    if list_options.author.is_some() {
        let author = list_options.author.as_ref().unwrap().to_owned().to_lowercase();
        extensions = extensions
            .into_iter()
            .filter(|e| e.author.as_ref().unwrap().to_lowercase().starts_with(&author))
            .collect();
    }
    if list_options.keyword.is_some() {
        let keyword = list_options.keyword.as_ref().unwrap().to_owned().to_lowercase();
        extensions = extensions
            .into_iter()
            .filter(|e| e.name.to_lowercase().starts_with(&keyword))
            .collect();
    }

    // TODO: sort
    // extensions = extensions.sort();
    let delimiter = if list_options.presentation == ListPresentation::Comma {","} else {"\n"};

    let na: Vec<_> = extensions
        .iter()
        .map(|e| e.name.as_str().to_owned())
        .collect();
    let str = na.join(delimiter);

    println!("{}", Green.paint(format!("{}", &str)));
}




/*
//let filteredData = filter_by_author(listOptions.author.as_ref().unwrap().to_owned(), &extensions);

fn filter_by_author(author: String, data: &Vec<Extension>) -> Vec<&Extension> {
    println!("Filter by author *{}*", author);
    return data
        .iter()
        .filter(|e| e.author.as_ref().unwrap().eq_ignore_ascii_case(&author))
        .collect();
}
 */

//let filtered: Vec<Extension> = data.extensions.iter().filter_map(|f| *f.name == "").collect();
//let finally_working_save_this = &listOptions.author.to_owned().unwrap_or_else(||"...".to_string());
//let x = listOptions.author. or_else("");
