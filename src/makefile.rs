use std::fs;
use regex::Regex;
use globwalk::DirEntry;
use lazy_static::lazy_static;

use crate::common::yes_or_no;
use ansi_term::Colour::*;

lazy_static! {
    static ref LIB_SOURCE_REGEX: Regex = Regex::new(r"^-(lib|source)").unwrap();
}

fn get_last_source_and_last_lib_lines(lines: &Vec<String>) -> (usize, usize) {
    lines
        .iter()
        .enumerate()
        .fold((0, 0), |(mut last_lib, mut last_source), (idx, line)| {
            if let Some(captures) = LIB_SOURCE_REGEX.captures(line) {
                match &captures[1] {
                    "lib" => {
                        last_lib = idx;
                    }
                    "source" => {
                        last_source = idx;
                    }
                    _ => {}
                }
            }
            (last_lib, last_source)
        })
}

// TODO: make sure the correct makefile comes in here
pub fn add_make_file_entry(_name: String, makefile: &DirEntry, makefile_entries: Vec<String>) {
    /*let text = format!("Add makefile entry to: {:?} ", makefile.path());
    if !yes_or_no(&text, true) {
        return;
    }*/

    let contents = fs::read_to_string(&makefile.path()).expect("Could not read the makefile");

    let mut lines: Vec<String> = contents
        .lines()
        .map(|s| s.to_string())
        .collect();
    let mut diff_lines = lines.clone();

    let (mut last_lib_line, _) = get_last_source_and_last_lib_lines(&lines);
    last_lib_line += 1;

    let libs_binding = makefile_entries.to_owned();
    let libs: Vec<String> = libs_binding
        .iter()
        .filter(|x| x.to_owned().ends_with(".tl"))
        .map(|s| s.to_string())
        .collect();

    // Iterate all the makefile entries for -lib and add to lines

    for lib in libs {
        let prep = format!("-lib {}", lib).to_string();
        if !contents.contains(&prep) {
            let colorized = format!("{}", Green.paint(&prep).to_owned().to_string());
            diff_lines.insert(last_lib_line, colorized);
            lines.insert(last_lib_line, prep);
            last_lib_line += 1;    
        }
    }

    let (_, mut last_source_line) = get_last_source_and_last_lib_lines(&lines);
    last_source_line += 1;
    let sources = libs_binding.iter().filter(|x| x.to_owned().ends_with(".t"));
    for source in sources {
        let prep = format!("-source {}", source).to_string();
        if !contents.contains(&prep) {
            let colorized = format!("{}", Green.paint(&prep).to_owned().to_string());
            diff_lines.insert(last_source_line, colorized);
            lines.insert(last_source_line, prep);
            last_source_line += 1;
        }
    }

    // Compiled changes 
    let makefile_changed = lines.join("\n");

    let trimmed_original = &contents.trim()
        .split(' ')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if makefile_changed.eq(&trimmed_original.to_string()) {
        // println!("{}", Yellow.paint(format!("[No changes needed in the makefile]")).to_string());
        return;
    }

    println!("{}", Yellow.paint(format!("Makefile suggested contents:")).to_string());
    print!("{}\n\n", diff_lines.join("\n"));


    if yes_or_no("Apply above changes?", true) {
        if makefile.path().exists() {
            fs::write(&makefile.path(), makefile_changed).expect(
                "Makefile could not be found. Skipping. "
            );
            println!("Changes applied");
        }
    } else {
        println!("No changes applied");
    }
    println!("\n");
    // TODO: save to file
}
