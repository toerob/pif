use std::fs;
extern crate ansi_term;
use ansi_term::Colour::*;
use crate::model::{Extensions, Extension};

pub fn install_extensions(name: &Vec<String>) -> () {
  let extension_data_str = fs::read_to_string("./extensions.json").unwrap();
  let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();
  let installable_extensions: Vec<Extension> = data
      .extensions
      .into_iter()
      .filter(|e| name.contains(&e.name))
      .collect();

  if installable_extensions.is_empty() {
      println!(
          "{}",
          Red.paint(format!(
              "No extension(s) found by the name: \"{}\"",
              &name.join(", ")
          ))
      );
      return;
  }
  installable_extensions.iter().for_each(|extension| {
      //let url = extension.url.as_ref().unwrap().as_str();
      println!("{}",Green.paint(format!("data = {}",extension.url.as_ref().unwrap().as_str())));
      //let resp = reqwest::blocking::get(url).unwrap();
      //println!("{:#?}", resp);
  });
}
