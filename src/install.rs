use ansi_term::Colour::*;
use git2::{ErrorCode, Repository};
use sublime_fuzzy::{FuzzySearch, Scoring};
use std::{
    fs::{self, File},
    io::{Cursor, Write},
    path::{Path},
};

use crate::{args::{GlobalOptions, Color}, makefile::add_make_file_entry};
use crate::model::{Extension, Extensions};

pub fn install_extensions(names: &Vec<String>, global_options: &GlobalOptions) -> () {
    let lower_case_names: Vec<String> = names.iter().map(|n|n.to_lowercase()).collect();
    let extension_data_str = fs::read_to_string("./extensions.json").unwrap();
    let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();
    
    let installable_extensions: Vec<Extension> = data.extensions.clone().into_iter()
        .filter(|e| lower_case_names.contains(&e.name.to_lowercase()))
        .collect();


    if installable_extensions.is_empty() {
        println!(
            "{}",
            Red.paint(format!(
                "No extension(s) found by the name: \"{}\"",
                &names.join(", ")
            ))
        );
        
        let mut all_results: Vec<String> = Vec::new();
        for name in names.iter() {
            data.extensions.clone().into_iter()
                .filter(|e| 
                    FuzzySearch::new(&name, &e.name.to_lowercase())
                    .case_insensitive()
                    .best_match()
                    .is_some())
                .map(|e|e.name)
                .for_each(|local_result| all_results.push(local_result));
        };
        if !all_results.is_empty() {
            print!("You may have meant to type: ");
            println!("{}", Yellow.paint(format!("{} ", all_results.join(", "))));
        }
        return;
    }

    //type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

    // TODO: ensure the extension folder exists or is created first

    installable_extensions.iter().for_each(|extension| {
        let library_path = "./libs"; // TODO: overridable set via installOptions

        let os_path = Path::new(library_path).join(&extension.name);
        let path = os_path.to_str().unwrap();

        let url = extension.url.as_ref().unwrap().as_str();

        let result: Vec<&str> = url.matches(".git").collect();
        let is_git_repo = !result.is_empty();

        let use_colors = if Color::Never == global_options.color {false} else {true};


        if !is_git_repo {
            // Regular ifarchive procedure
            let response = reqwest::blocking::get(url).expect("Request did fail");
            let file_data = response.bytes().expect("Bytes are invalid");
            let file_extension = extension.ext.as_ref().to_owned().unwrap();
            if file_extension == "zip" {
                let target_dir = Path::new(os_path.as_path());
                zip_extract::extract(Cursor::new(file_data), &target_dir, true)
                    .expect("Failed extract file. ");
            } else {
                let file_name: String = path.to_owned() + "." + file_extension;
                let mut file = File::create(file_name).expect("failed to create file. ");
                file.write_all(&file_data)
                    .expect("Failed to write to binary file. ");
            }
            let text = format!(" ==> {} INSTALLED", &extension.name);
            if use_colors {
                println!("{}", Green.paint(text));
            }else {
                println!("{}", text);
            }

            // TODO: add_make_file_entry();
            
            return;
        }

        // Clone repository procedure

        // 1. First check if repository already exists
        match Repository::open(path) {
            Ok(_) => {
                println!("Repository already exists! TODO: update repository");
                return;
            }
            Err(e) => {
                // If repository isn't found, don't panic!
                if ErrorCode::NotFound == e.code() {
                    //println!("{:?}", e.code());
                } else {
                    // If due to other error, let's panic!
                    panic!("failed to open: {}", e);
                }
            }
        };

        // 2. If repository doesn't exist. Clone it into the folder of {path}


        /*
        // This below is for animation purpose, spawn a thread to print to stdout every nth millisecond

        print!("Installing {}, progress: ", &extension.name);
        // Shared progress counter
        let progress = Arc::from(Mutex::from(0 as usize));

        // TODO: generalize and reuse for regular installation as well (IF using this at all.)
        // copy of progress counter for animation thread
        let progress_t = progress.clone();
        let animation_process_handle = spawn(move || {
            stdout().flush().unwrap();
            loop {
                {
                    let mut val = progress_t.lock().unwrap();
                    if *val >= 100 {
                        stdout().flush().unwrap();
                        break;
                    }
                    *val = *val + 1;
                    stdout().write_all(".".as_bytes()).unwrap();
                    stdout().flush().unwrap();
                }
                sleep(Duration::from_millis(40));
            }
        }); 
        */

        // 3. Clone the repository
        match Repository::clone(url, path) {
            Ok(_) => {
                //println!(" DONE!");
                //let text = format!("Installing \"{}\" in folder: {}", &extension.name, &path);
                let text = format!(" ==> {} INSTALLED", &extension.name);
                if use_colors {
                    println!("{}", Green.paint(text));
                }else {
                    println!("{}", text);
                }
            }
            Err(e) => {
                /*
                // Make sure to stop the thread before panicking.
                *progress.lock().unwrap() = 100; // Trigger 100% and cause the animation to end
                animation_process_handle.join().unwrap();
                 */
                panic!("failed to clone repository: {}", e)
            }
        };

        /*
        // Make sure to stop the thread.
        *progress.lock().unwrap() = 100; // Trigger 100% and cause the animation to end
        animation_process_handle.join().unwrap();
         */
    });
}

/*

let repo = match Repository::open(path) {
       Ok(repo) => {
           /*let statuses = match repo.statuses(None) {
               Ok(statuses) => {
                   statuses.iter().for_each(|status| {
                       println!("STATUSES OK!");
                       //status.status()
                   });
                   //let status = statuses.get(0);
                   print!("{}", statuses.len());
                   //statuses.len();
               }
               Err(e) => panic!("failed to fetch statuses: {}", e),
           };*/

           /*for entry in statuses.iter() {

               let status = entry.status();
               //if status.intersects(INTERESTING) {
               if let Some(path) = entry.path() {
                   let path = workdir.join(path);
                   interesting_statuses.insert(path, status);
               }
               //}
           }*/
           print!("OK!");
           //repo;

       },
       Err(e) => panic!("failed to open: {}", e),

*/
