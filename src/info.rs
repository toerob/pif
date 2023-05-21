use crate::args;
use ansi_term::Colour::*;
use std::{
    fs::{self},
};

use crate::model::{Extensions};
use crate::{
    detect::{detect_system, get_extension_path},
};

pub fn extensions_info(names: &[String], global_options: &args::GlobalOptions) {
    if names.len() == 0 {
        println!(
        "{}",Red.paint(format!("No packages specified. Command usage examples: \n  \"ifp install abc \"\n  \"ifp install abc def\""))
    );
        return;
    }
    let (system_type, _) = detect_system();
    println!(
        "{}",
        Yellow
            .paint(format!("System: {:?}", system_type))
            .to_string()
    );

    let file_path = get_extension_path(system_type);

    let extension_data_str = fs::read_to_string(file_path).unwrap();
    let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();

    let lowercase_names: Vec<String> = names
        .to_owned()
        .into_iter()
        .map(|it| it.to_lowercase())
        .collect();

    let extensions_info = data
        .extensions
        .iter()
        .filter(|ext| lowercase_names.contains(&ext.to_owned().name.to_lowercase()));

    for ele in extensions_info {
      println!("Extension name: {}\nby {} \nDescription: {}\n", &ele.name,  &ele.author.as_ref().unwrap(),  &ele.desc.as_ref().unwrap());


      if ele.versions.to_owned().len() > 0 {
        println!("Versions: ");
        for version in &ele.versions {
          println!(" {}\n last modified: {}\n", &version.url.as_ref().unwrap(), &version.last_modified.as_ref().unwrap());
        }
  
      }

    }
}
