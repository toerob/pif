use std::{ io::{ stdout, Write }, process::exit };
use std::{ fs::{ self, File }, io::{ Cursor }, path::Path };
use ansi_term::Colour::*;

use online::check;
use crate::{ args::{ GlobalOptions, Color }, color::{ print_success_msg } };
use git2::{
    Error,
    ErrorCode,
    Repository,
    FetchOptions,
    StatusOptions,
    MergeOptions,
    RemoteCallbacks,
    Remote,
    Cred,
};

use std::path::PathBuf;

use dirs_next::home_dir;

fn get_or_create_repo_dir(subdir: &str) -> Result<PathBuf, std::io::Error> {
    // Hämta hemkatalog och lägg till underkatalog
    let home_dir = home_dir().expect("Could not determine home directory");
    let repo_dir = home_dir.join(subdir);

    // Skapa katalogen om den inte finns
    if !repo_dir.exists() {
        fs::create_dir_all(&repo_dir.as_path())?;
    }

    Ok(repo_dir)
}

fn clone_or_pull_repo(repo_url: &str, repo_path: &PathBuf) -> Result<(), Error> {
    if repo_path.join(".git").exists() {
        println!("Repository exists. Performing 'git pull' on {}...", repo_path.display());
        let repo = Repository::open(repo_path)?;

        // Ställ in callbacks för autentisering
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::default() // Använder systemets standard-autentisering
        });

        // Konfigurera fetch-alternativ
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Hämta från remote
        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&["refs/heads/master"], Some(&mut fetch_options), None)?;

        // Hämta FETCH_HEAD och skapa AnnotatedCommit
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

        // Kontrollera om merge behövs
        let analysis = repo.merge_analysis(&[&fetch_commit])?;
        if analysis.0.is_up_to_date() {
            println!("Already up-to-date.");
        } else if analysis.0.is_fast_forward() {
            println!("Fast-forwarding...");
            let mut reference = repo.find_reference("refs/heads/master")?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            repo.set_head("refs/heads/master")?;
            repo.checkout_head(Some(&mut git2::build::CheckoutBuilder::default()))?;
        } else if analysis.0.is_normal() {
            println!("Performing a normal merge...");
            repo.merge(&[&fetch_commit], None, None)?;

            // Kontrollera merge-konflikter
            if repo.index()?.has_conflicts() {
                println!("Merge conflicts detected!");
                // Hantera konflikter här (avsluta eller abortera).
            } else {
                println!("Merge completed without conflicts. Cleaning up...");
                repo.checkout_head(Some(&mut git2::build::CheckoutBuilder::default()))?;
                // Merge avslutad utan konflikter, så vi gör inget ytterligare.
            }
        } else {
            println!("Unknown merge state. Aborting merge...");
            repo.checkout_head(Some(&mut git2::build::CheckoutBuilder::default()))?;
        }
    } else {
        println!("Cloning repository from {} to {:?}", repo_url, repo_path);
        Repository::clone(repo_url, repo_path)?;
        println!("Repository cloned successfully.");
    }
    Ok(())
}

// TODO: add colors here as well
pub fn update_extensions3(global_options: &GlobalOptions, workspace_folder: &str) {
    let repository_url = "https://github.com/toerob/t3cartographer"; // TODO: Add public repository here instead of this placeholder
    let repository_main_branch = "master"; // TODO: rename to main when the public repository exists

    let repo_dir = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("ifp")
        .join("repo")
        .clone();

    let repo_dir_str = repo_dir.as_path().to_str().clone().unwrap();

    match get_or_create_repo_dir(&repo_dir_str) {
        Ok(repo_path) => {
            if let Err(e) = clone_or_pull_repo(repository_url, &repo_path) {
                eprintln!("Error: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to create repository directory: {}", e),
    }
}

// TODO: add colors here as well
pub fn update_extensions2(global_options: &GlobalOptions, workspace_folder: &str) {
    if !check(Some(5)).is_ok() {
        println!("No internet connection. Aborting. ");
        exit(0);
    }

    let use_colour = Color::Never != global_options.color;
    let success_color = match Color::Never != global_options.color {
        true => Some(Green),
        false => None,
    };

    let repository_url = "https://github.com/toerob/t3cartographer"; // TODO: Add public repository here instead of this placeholder
    let repository_main_branch = "master"; // TODO: rename to main when the public repository exists
    if !ensure_open(workspace_folder) {
        //println!("{}", colorize_message(success_color,format!(" ==> Downloading latest changes from: {} ", repository_url)));
        stdout().flush().unwrap();
        Repository::clone(repository_url, workspace_folder).expect(
            "failed to clone repository: {}"
        );
    }

    let repo_dir = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("ifp")
        .join("repo");

    // TODO: check latest commit at origin

    if repo_dir.exists() {
        println!("Repository already exists at {:?}", repo_dir);

        // TODO: check if a change is needed. Compare origin with local

        fs::remove_dir_all(&repo_dir).unwrap();
        println!("Removing it");
    }

    let os_path = Path::new(repository_url); //.join(&repo_dir);
    let path = os_path.to_str().unwrap();

    match Repository::open(path) {
        Ok(repo) => {
            println!("Updating repository...");

            // Hämta remote (som oftast är 'origin')
            let mut remote = repo.find_remote("origin").unwrap();

            // Konfigurera callbacks för autentisering
            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(|_url, _username_from_url, _allowed_types| { Cred::default() });

            // Ställ in fetch-alternativ
            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(callbacks);

            // Fetch från remote
            remote.fetch(&["refs/heads/master"], Some(&mut fetch_options), None).unwrap();

            // Uppdatera lokala branch
            let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
            repo.set_head(fetch_head.name().unwrap()).unwrap();
            repo.checkout_head(None).unwrap();

            /*
            TODO:
            let our_commit = find_last_commit(&repo).expect("FAIL");
            println!("Branch updated to match remote.");
            let their_commit = fetch_head.peel_to_commit().expect("FAIL");
            let _index = repo
                    .merge_commits(&our_commit, &their_commit, Some(&MergeOptions::new())).expect("FAIL");
                    */

            // TODO:
            /* let modifiedEntries: Vec<String> = statuses.iter()
                .filter(|status|status.status().is_index_modified())
                .map(|status| &status.path().unwrap().to_string())
                .collect();*/

            //.clone()
            //.into_iter()

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
    }

    print_success_msg(
        use_colour,
        format!(" ==> Downloading latest changes from: {} \n", repository_url)
    );

    //println!("Cloning repository from {} to {:?}", repository_url, repo_dir);
    Repository::clone(repository_url, repo_dir).expect("Clone failed");
    print_success_msg(use_colour, format!("UPDATED\n"));

    //pull_latest(global_options, workspace_folder, &repository_main_branch.to_string());
}

// TODO: add colors here as well
pub fn update_extensions(global_options: &GlobalOptions, workspace_folder: &str) {
    if !check(Some(5)).is_ok() {
        println!("No internet connection. Aborting. ");
        exit(0);
    }

    let use_colour = Color::Never != global_options.color;
    /*let success_color = match Color::Never != global_options.color {
        true => Some(Green),
        false => None,
    };*/

    let repository_url = "https://github.com/toerob/t3cartographer"; // TODO: Add public repository here instead of this placeholder
    let repository_main_branch = "master"; // TODO: rename to main when the public repository exists
    if !ensure_open(workspace_folder) {
        print_success_msg(
            use_colour,
            format!(" ==> Downloading latest changes from: {} \n", repository_url)
        );
        //println!("{}", colorize_message(success_color,format!(" ==> Downloading latest changes from: {} ", repository_url)));
        stdout().flush().unwrap();
        Repository::clone(repository_url, workspace_folder).expect(
            "failed to clone repository: {}"
        );
    }

    pull_latest(global_options, workspace_folder, &repository_main_branch.to_string());
}

fn ensure_open(workspace_folder: &str) -> bool {
    match Repository::open(workspace_folder) {
        Ok(_) => true,
        Err(e) => {
            if ErrorCode::NotFound == e.code() {
                println!("Settings are missing. ");
            } else {
                panic!("failed to open: {}", e); // If due to other error than NotFound, let's panic!
            }
            false
        }
    }
}

fn pull_latest(
    global_options: &GlobalOptions,
    workspace_folder: &str,
    remote_branch: &String
) -> () {
    let use_colour = Color::Never != global_options.color;

    /*let success_color = match Color::Never != global_options.color {
        true => Some(Green),
        false => None,
    };

    //print!("{}", colorize_message(success_color, String::from(" ==> Fetching latest changes")));*/

    print_success_msg(use_colour, String::from(" ==> Fetching latest changes\n"));

    let repo = Repository::open(workspace_folder).expect("Failed to open repository");
    let mut origin = repo.find_remote("origin").expect("Find origin failed");

    let mut fetch_options = FetchOptions::new();
    //origin.fetch(&[remote_branch], Some(&mut fetch_options), None).expect("Fetch origin failed");
    origin
        .fetch(&["refs/heads/main"], Some(&mut fetch_options), None)
        .expect("Fetch origin failed");
    print!("Fetch complete");

    let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
    repo.set_head(fetch_head.name().unwrap()).unwrap();

    repo.checkout_head(None).unwrap();
    println!("Branch updated to match remote.");

    /*
    let fetch_head = repo.find_reference("FETCH_HEAD").expect("msg");
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head).expect("...");
        
    let analysis = repo.merge_analysis(&[&fetch_commit]).expect("...");
    
    if analysis.0.is_up_to_date() {
        println!(" Done! (No changes - you are up to date)");
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", remote_branch);
        let mut reference = repo
            .find_reference(&refname)
            .expect("Could not find reference branch");
        reference
            .set_target(fetch_commit.id(), "Fast-Forward")
            .expect("Could not set target");
        repo.set_head(&refname).expect("Could not set head");
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .expect("Could not checkout head");
        println!(" Done! Extension master list is now updated.");
    } else {
        //let x = Error::from_str("Fast-forward only!");
        panic!("Fast-forward only!");
    }
    */
}

/*
pub fn find_last_commit(repo: &Repository) -> Result<Commit, RepoError> {
    let obj = repo.head().expect("NAH").resolve().expect("NO").peel(ObjectType::Commit)?;
    match obj.into_commit() {
        Ok(c) => Ok(c),
        _ => Err(RepoError::new("commit error")),
    }
} */

/*
// TODO: Check statuses
let mut statuses_options = StatusOptions::new();
let statuses = repo.statuses(Some(&mut statuses_options)).unwrap();
statuses.iter()
    .filter(|status|status.status().is_index_modified())
    .for_each(|status| {
        println!("Modified entry: {}", &status.path().unwrap());
    });
*/

// TODO: detta steg lyckas inte riktigt.

// https://stackoverflow.com/questions/9237348/what-does-fetch-head-in-git-mean
