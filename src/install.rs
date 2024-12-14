use ansi_term::Colour::*;
use git2::{ ErrorCode, Repository, FetchOptions, StatusOptions, RemoteCallbacks, Remote, Cred };

use online::check;
use std::{ fs::{ self, File }, io::{ Cursor, Write }, path::Path, process::exit };
use sublime_fuzzy::FuzzySearch;

use crate::{
    args::InteractiveFictionSystem,
    color::{ print_warning_msg, print_success_msg },
    model::{ Extension, Extensions },
};
use crate::{
    args::{ Color, GlobalOptions, InstallOptions },
    detect::{ detect_system, get_extension_path },
    makefile::add_make_file_entry,
    gitops::{ get_or_create_repo_dir, clone_or_pull_repo },
    update::{ update_extensions },
};

use dirs_next::data_dir;

pub fn install_extensions(
    names: &Vec<String>,
    install_options: &InstallOptions,
    global_options: &GlobalOptions,
    update_needed: bool
) -> () {

    if update_needed {
        update_extensions(global_options);
    }

    let use_colours = Color::Never != global_options.color;

    /*// TODO: 5 'r placeholder?
    if !check(Some(5)).is_ok() {
        print_warning_msg(use_colours, "No internet connection. Aborting. \n".to_string());
        exit(0);
    }*/

    if names.len() == 0 {
        println!(
            "{}",
            Red.paint(
                format!(
                    "No packages specified. Command usage examples: \n  \"pif install abc \"\n  \"pif install abc def\""
                )
            )
        );
        return;
    }

    let (system_type, makefile) = if global_options.system == InteractiveFictionSystem::Auto {
        detect_system()
    } else {
        (global_options.system.clone(), None)
    };

    println!("{}", Yellow.paint(format!("System: {:?}", system_type)).to_string());
    let file_path_end = get_extension_path(system_type);

    // TODO: extract version part from name@version if present
    //let lower_case_names: Vec<String> = names.iter()
    //    .map(|n| n.split("@").to_lowercase()).collect();

    let lower_case_names: Vec<String> = names
        .iter()
        .map(|n| n.to_lowercase())
        .collect();


    let file_path = dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo")
        .join(file_path_end)
        .clone();

    let file_path_str = file_path.as_path().to_str().clone().unwrap();

    print!("Trying: {}", file_path_str);

    let extension_data_str = fs::read_to_string(file_path).unwrap();
    let data: Extensions = serde_json::from_str(&extension_data_str).unwrap();

    let installable_extensions: Vec<Extension> = data.extensions
        .clone()
        .into_iter()
        .filter(|e| lower_case_names.contains(&e.name.to_lowercase()))
        .collect();

    if installable_extensions.is_empty() {
        print_warning_msg(
            use_colours,
            format!("No extension(s) found by the name: \"{}\"\n", &names.join(", "))
        );

        let mut all_results: Vec<String> = Vec::new();
        for name in names.iter() {
            data.extensions
                .clone()
                .into_iter()
                .filter(|e| {
                    FuzzySearch::new(&name, &e.name.to_lowercase()) // TODO: should not be fuzzy during intall?
                        .case_insensitive()
                        .best_match()
                        .is_some()
                })
                .map(|e| e.name)
                .for_each(|local_result| all_results.push(local_result));
        }
        if !all_results.is_empty() {
            print!("You may have meant to type: ");
            println!("{}", Yellow.paint(format!("{} ", all_results.join(", "))));
        }
        return;
    }

    //type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

    // TODO: ensure the extension folder exists or is created first

    /*
    if install_options.installation_directory.is_some() {
        let dir = install_options.installation_directory.as_ref().unwrap().to_owned();
    } else {
        println!("\nNo default value for!!!****");
    }*/

    let library_path = install_options.installation_directory.as_ref().unwrap(); // TODO: overridable set via installOptions

    // Ensure directory exists:
    if !std::path::Path::new(&library_path).exists() {
        match fs::create_dir_all(library_path) {
            Ok(p) => {}
            Err(e) => {
                print_warning_msg(
                    use_colours,
                    format!("Could not create directory {}, beacuse: {} ", library_path, e)
                );
                return;
            }
        }
    }

    installable_extensions.iter().for_each(|extension| {
        let os_path = Path::new(library_path).join(&extension.name);
        let path = os_path.to_str().unwrap();

        let latest_version = extension.versions.get(0).unwrap();

        let url = latest_version.url.as_ref().unwrap().as_str();
        let branch_name = latest_version.branch.as_ref().unwrap();

        let result: Vec<&str> = url.matches(".git").collect();
        let is_git_repo = !result.is_empty();

        let use_colors = if Color::Never == global_options.color { false } else { true };

        if !is_git_repo {
            // Regular ifarchive procedure
            let response = reqwest::blocking::get(url).expect("Request did fail");
            let file_data: hyper::body::Bytes = response.bytes().expect("Bytes are invalid");
            let file_extension = latest_version.ext.as_ref().to_owned().unwrap();
            if file_extension == "zip" {
                let target_dir = Path::new(os_path.as_path());
                zip_extract
                    ::extract(Cursor::new(file_data), &target_dir, true)
                    .expect("Failed extract file. ");
            } else {
                let file_name: String = path.to_owned() + "." + file_extension;
                let mut file = File::create(file_name).expect("failed to create file. ");
                file.write_all(&file_data).expect("Failed to write to binary file. ");
            }
            let text = format!(" ==> {} INSTALLED", &extension.name);
            if use_colors {
                println!("{}", Green.paint(text));
            } else {
                println!("{}", text);
            }

            if makefile.is_some() && latest_version.makefile_entries.is_some() {
                
                add_make_file_entry(
                    extension.name.clone(),
                    makefile.as_ref().unwrap(),
                    latest_version.makefile_entries.as_ref().unwrap().to_owned()
                );
            }
            // TODO: add_make_file_entry() for both, not just non-git;
            return;
        }

        // IT is a GIT repo, clone or update it:
        // TODO: make branch name adaptable from the json format
        let branch_head_name = format!("refs/heads/{}", branch_name);
        
        match get_or_create_repo_dir(&path) {
            Ok(repo_path) => {
                if let Err(e) = clone_or_pull_repo(url, &branch_head_name, &repo_path) {
                    print_warning_msg(use_colours, format!("Error: {}\n", e));
                }
                print_success_msg(
                    use_colours,
                    format!("Extension installed into {}\n", repo_path.display())
                );
            }
            Err(e) => { eprintln!("Failed to create repository directory: {}", e) }
        }
    });
}

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

        // Make sure to stop the thread.
        *progress.lock().unwrap() = 100; // Trigger 100% and cause the animation to end
        animation_process_handle.join().unwrap();
         */
