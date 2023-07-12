use std::fs;

use globwalk::DirEntry;

use crate::model::{Extension, Version};

pub fn add_make_file_entry(name: String, makefile: &DirEntry, makefile_entries: Vec<String>) {
  print!("Add makefile entry to: {:?} ? (y/n, default: y): ", makefile.path());

  let contents = fs::read_to_string(makefile.path().clone())
    .expect("Could not read the makefile");



  let lines: Vec<&str> = contents.lines().collect();
  print!("\nMakefile entries:\n");
  makefile_entries.iter()
    .for_each(|x| print!("{x}"));

  println!();

  // TODO: locate suitable place within contents and concat the makefile_entries there

  println!("Makefile contents:");
  print!("{contents}");

} 