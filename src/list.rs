extern crate ansi_term;
extern crate clap;
extern crate serde;

use crate::args::{Color, GlobalOptions, SortProperty};
use ansi_term::Colour::*;
use args::{ListOptions, ListPresentation};
use model::Extensions;
use std::fs;
use sublime_fuzzy::FuzzySearch;

#[warn(unused_attributes)]
pub fn list_extensions(list_options: &ListOptions, global_options: &GlobalOptions) -> () {
    let extension_data_str = fs::read_to_string("./extensions.json").unwrap();
    let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();
    let mut extensions = data.extensions;

    if list_options.author.is_some() {
        let author = list_options
            .author
            .as_ref()
            .unwrap()
            .to_owned()
            .to_lowercase();
        extensions = extensions
            .into_iter()
            .filter(|e| {
                FuzzySearch::new(&author, &e.author.as_ref().unwrap().to_lowercase())
                    .case_insensitive()
                    .best_match()
                    .is_some()
            })
            .collect();
    }
    if list_options.keyword.is_some() {
        
        let keyword = list_options
            .keyword
            .as_ref()
            .unwrap()
            .to_owned()
            .to_lowercase();

        extensions = extensions
            .into_iter()
            .filter(|e| {
                FuzzySearch::new(&keyword, &e.name.to_lowercase())
                    .case_insensitive()
                    .best_match()
                    .is_some()
            })
            .collect();
    }
    
    // TODO: implement OrderingDirection -> SortOrderDir

    if SortProperty::Name == list_options.sort_property {
        extensions.sort_by_key(|e|e.to_owned().name);
    } else if SortProperty::Author == list_options.sort_property {
        extensions.sort_by_key(|e|e.to_owned().author);
    } else if SortProperty::Date == list_options.sort_property {
        extensions.sort_by_key(|e|e.to_owned().last_modified);
    }



    

    let delimiter = if list_options.presentation == ListPresentation::Comma {
        ","
    } else {
        "\n"
    };

    let na: Vec<_> = extensions
        .iter()
        .map(|e| create_presentation(e, global_options))
        .collect();

    let str = na.join(delimiter);
    println!("{}", str);

}

fn create_presentation(e: &crate::model::Extension, global_options: &GlobalOptions) -> String {
    let use_colors = if Color::Never == global_options.color {
        false
    } else {
        true
    };
    let verbosity_level = global_options.verbose.unwrap();
    let name = if use_colors {
        Green
            .paint(format!("{}", e.name.as_str().to_owned()))
            .to_string()
    } else {
        e.name.as_str().to_owned()
    };

    return match verbosity_level {
        1 => name,
        2 => {
            name + " ("
                + e.last_modified.as_ref().unwrap().to_owned().as_str()
                + ") "
                + " by "
                + e.author.as_ref().unwrap().to_owned().as_str()
        }
        _ => {
            name + " ("
                + e.last_modified.as_ref().unwrap().to_owned().as_str()
                + ") "
                + " by "
                + e.author.as_ref().unwrap().to_owned().as_str()
                + " "
                + e.desc.as_ref().unwrap().to_owned().as_str()
        }
    };
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
