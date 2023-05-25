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

        //let iter = extensions_info.enumerate();
    //let nr: usize = extensions_info.count();
    for (_, ele) in extensions_info.enumerate() {
      println!("Extension name: {}\nby {} \nDescription: {}\n", &ele.name,  &ele.author.as_ref().unwrap(),  &ele.desc.as_ref().unwrap());


      if ele.versions.to_owned().len() > 0 {
        println!("Versions: ");
        let mut sorted_versions =  ele.clone();

        
        sorted_versions.versions.sort_by_key(|e|e.to_owned().version);

        let total = sorted_versions.versions.len()-1;

        for (idx, version) in sorted_versions.versions.into_iter().enumerate() {
            
            let latest = if (idx ==  total) { "(LATEST)" } else { "" };
            
          println!("{} {}\n {}\n last modified: {}\n", &version.version.as_ref().unwrap(), latest, &version.url.as_ref().unwrap(), &version.last_modified.as_ref().unwrap());
        }
  
      }

    }
}
