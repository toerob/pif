use std::fs;
use regex::Regex;
use std::error::Error;
use globwalk::DirEntry;
use lazy_static::lazy_static;

lazy_static! {
    static ref LIB_SOURCE_REGEX: Regex = Regex::new(r"^-(lib|source)").unwrap();
}

fn get_last_source_and_last_lib_lines(lines: &[&str]) -> (usize, usize) {
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
    print!("Add makefile entry to: {:?} ? (y/n, default: y): ", makefile.path());

    print!("\nMakefile entries:\n");

    let contents = fs
        ::read_to_string(makefile.path())
        .expect("Could not read the makefile");

    let mut lines: Vec<&str> = contents.lines().collect();

    let (mut last_lib_line, mut last_source_line) = get_last_source_and_last_lib_lines(&lines);

    println!("Last -lib line: {}", last_lib_line);
    println!("Last -source line: {}", last_source_line);

    println!("[last lib line: {}, and last source line: {}]\n", last_lib_line, last_source_line);
    println!(
        "[last lib word: {}, and last source word: {}]\n",
        lines[last_lib_line],
        lines[last_source_line]
    );
    // TODO: locate suitable place within contents and concat the makefile_entries there

    // TODO: decide for each entry if it is a lib or source row
    // if a lib, Locate the last -lib and use that as offset
    // if a source, Locate the last -source and use that as offset

    let libs_binding = makefile_entries.to_owned();

    let libs: Vec<String> = libs_binding
        .iter()
        .filter(|x| x.to_owned().ends_with(".tl"))
        .map(|s| s.to_string())
        .collect();

    for lib in libs {
        println!("** [ADDING LIB: {}] ** \n", lib);
        let together = format!("-lib {}", lib);
        //TODO: lines.insert(last_lib_line, together);
        last_lib_line += 1;
    }

    (_, last_source_line) = get_last_source_and_last_lib_lines(&lines);

    let sources = libs_binding.iter().filter(|x| x.to_owned().ends_with(".t"));
    for source in sources {
        println!("** [ADDING SOURCE: {}] ** \n", source);
        lines.insert(last_source_line, source);
        last_source_line += 1;
    }

    println!("Makefile suggested contents:");
    print!("{}", lines.join("\n"));

    println!("Apply? y/n (n):");
}
