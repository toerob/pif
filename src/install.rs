use ansi_term::Colour::*;

use std::{ collections::HashMap, fs::{ self, File }, io::{ Cursor, Write }, path::Path, process::exit };
use sublime_fuzzy::FuzzySearch;
use std::env::{current_dir};

use crate::{
    args::InteractiveFictionSystem,
    color::{ print_success_msg, print_warning_msg },
    model::{ Extension, Extensions, Version },
};
use crate::{
    args::{ Color, GlobalOptions, InstallOptions },
    detect::{ detect_system, get_extension_path },
    makefile::add_make_file_entry,
    gitops::{ get_or_create_repo_dir, clone_or_pull_repo, latest_semver_tag, checkout_tag },
    update::{ update_extensions },
    db::{ get_or_create_table, record_installation },
};

pub fn install_extensions(
    names: &Vec<String>,
    //install_version: &String,
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
    if makefile.is_some() {
        println!(
            "{}",
            Yellow.paint(
                format!(
                    "Makefile detected: {:?}",
                    makefile.as_ref().unwrap().to_owned().path().display()
                )
            ).to_string()
        );
    }

    let file_path_end = get_extension_path(system_type);

    // TODO: extract version part from name@version if present
    //let lower_case_names: Vec<String> = names.iter()
    //    .map(|n| n.split("@").to_lowercase()).collect();

    /*let lower_case_names: Vec<String> = names
        .iter()
        .map(|n| n.to_lowercase())
        .collect();*/

    let name_version_map: HashMap<String, String> = names
        .iter()
        .map(|x|{
            let lowercased = x.to_lowercase();
            let splitted: Vec<&str> = lowercased.split(':').collect();
            if splitted.len() == 2 {
                (splitted[0].to_string(), splitted[1].to_string())
            } else {
                (splitted[0].to_string(), "LATEST".to_string())
            }
        })
        .collect();

    
    let lower_case_names : Vec<String> = name_version_map
        .keys()
        .map(|x|x.to_string())
        .collect();

    let file_path = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo")
        .join(file_path_end)
        .clone();

    let _file_path_str = file_path.as_path().to_str().clone().unwrap();
    print!("Trying: {}", &_file_path_str);

    let extension_data_str = fs::read_to_string(file_path).unwrap();
    let data: Extensions = serde_yaml::from_str(&extension_data_str).unwrap();

    for warning in data.validate() {
        print_warning_msg(use_colours, format!("Schema warning: {}\n", warning));
    }

    let installable_extensions: Vec<Extension> = data.extensions
        .clone()
        .into_iter()
        .filter(|e| {
            lower_case_names.contains(&e.name.to_lowercase())            
        })
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
            Ok(_p) => {}
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
        let current_dir = current_dir().unwrap();
        let os_path = Path::new(&current_dir).join(library_path).join(&extension.name);
        let path = os_path.to_str().unwrap();

        println!("installation {path}");


        // TODO: 
        let found_matching_version: Option<Version>;
        let version_asked_for = name_version_map.get(&extension.name.to_lowercase()).unwrap(); //.unwrap_or_else('');
        
        //extension.versions.sort_by_key(|x|x.version);

        let mut versions = extension.versions.clone();

        if version_asked_for.to_lowercase() == "latest" {
            print!("Version asked for is 'LATEST'\n");
            versions.sort_by_key(|x| x.version.clone().unwrap_or(semver::Version::new(0, 0, 0)));
            found_matching_version = versions.last().cloned();
        } else if version_asked_for.to_lowercase() == "snapshot" {
            print!("Version asked for is 'SNAPSHOT'\n");
            let version_req = semver::VersionReq::parse("0.0.0-SNAPSHOT").unwrap();
            found_matching_version = versions
                .into_iter()
                .find(|x| x.version.as_ref().map_or(false, |v| version_req.matches(v)));
        } else {
            let version_req = semver::VersionReq::parse(version_asked_for)
                .expect("Not a compatible version format");
            found_matching_version = versions
                .into_iter()
                .find(|x| x.version.as_ref().map_or(false, |v| version_req.matches(v)));
        }


        /* 
        let text = match colour {
            Some(c) => format!("{}", c.paint(msg).to_owned()),
            None => msg.to_owned(),
        };*/

        match &found_matching_version {
            Some(x) => {
                let v = x.version.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "unknown".to_string());
                print!("Version asked for: {} found -> {}", version_asked_for, v);
            },
            None => {
                print!("No version found\n");
                exit(0);
            }
        }

        let latest_version = &found_matching_version.unwrap();
        //let latest_version = extension.versions.get(0).unwrap();

        let url = latest_version.url.as_ref().unwrap().as_str();
        let is_git_repo = latest_version.branch.is_some();

        let use_colors = if Color::Never == global_options.color { false } else { true };

        if !is_git_repo {
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

            add_data_record(&extension.name, &path, use_colors);

            let text = format!(" ==> {} installed into directory {}", &extension.name, &os_path.display());
            if use_colors { println!("{}", Green.paint(text)); } else { println!("{}", text); }

            if makefile.is_some() && latest_version.makefile_entries.is_some() {
                add_make_file_entry(
                    extension.name.clone(),
                    makefile.as_ref().unwrap(),
                    latest_version.makefile_entries.as_ref().unwrap().to_owned()
                );
            }
            return;
        }

        // Git repo: clone/pull, then resolve to latest release tag unless SNAPSHOT was requested.
        let branch_name = latest_version.branch.as_deref().unwrap_or("master");
        let branch_head_name = format!("refs/heads/{}", branch_name);
        let is_snapshot = version_asked_for.to_lowercase() == "snapshot";

        match get_or_create_repo_dir(&path) {
            Ok(repo_path) => {
                if let Err(e) = clone_or_pull_repo(url, &branch_head_name, &repo_path) {
                    print_warning_msg(use_colours, format!("Error: {}\n", e));
                    return;
                }

                if is_snapshot {
                    print_success_msg(use_colours, format!("Using branch tip ({})\n", branch_name));
                } else {
                    match git2::Repository::open(&repo_path)
                        .ok()
                        .and_then(|r| latest_semver_tag(&r).map(|t| (r, t)))
                    {
                        Some((repo, tag)) => {
                            match checkout_tag(&repo, &tag) {
                                Ok(_) => print_success_msg(use_colours, format!("Checked out release {}\n", tag)),
                                Err(e) => print_warning_msg(use_colours, format!("Could not checkout tag {}: {} — using branch tip\n", tag, e)),
                            }
                        }
                        None => print_success_msg(use_colours, format!("No release tags found, using branch tip ({})\n", branch_name)),
                    }
                }

                print_success_msg(use_colours, format!("Extension installed into {}\n", repo_path.display()));
            }
            Err(e) => {
                eprintln!("Failed to create repository directory: {}", e);
            }
        }

        add_data_record(&extension.name, &path, use_colors);

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

fn add_data_record(name: &str, path: &str, use_colors: bool) {
    // TODO:  egen metod och återanvänd
    match get_or_create_table() {
        Ok(conn) => {
            record_installation(&conn, &name, &path);
        }
        Err(e) => {
            print_warning_msg(
                use_colors,
                format!("Something failed when trying to access sqlite db: {}\n", e)
            );
        }
    }
}
