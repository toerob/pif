use std::{ io::{ stdout, Write }, process::exit };
use std::{ fs::{ self, File }, io::{ Cursor }, path::Path };
use ansi_term::Colour::*;

use online::check;
use crate::{ args::{ GlobalOptions, Color }, color::{ print_success_msg, print_warning_msg } };
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

use config::Config;
use std::collections::HashMap;

pub fn update_extensions3(global_options: &GlobalOptions, workspace_folder: &str) {
    let use_colour = Color::Never != global_options.color;
    let success_color = match Color::Never != global_options.color {
        true => Some(Green),
        false => None,
    };

    let settings = Config::builder()
        .add_source(config::File::with_name("Settings.yaml"))
        .build()
        .unwrap();

    let config = settings.clone().try_deserialize::<HashMap<String, String>>().unwrap();

    let repository_url = &settings.get_string("main_repository_url").unwrap();
    let repository_main_branch = &settings.get_string("main_repository_branch").unwrap();

    // println!("{:?}", &repository_url);
    // println!("{:?}", &repository_main_branch);

    let repo_dir = dirs_next
        ::data_dir()
        .expect("Could not determine data directory")
        .join("ifp")
        .join("repo")
        .clone();

    let repo_dir_str = repo_dir.as_path().to_str().clone().unwrap();

    match get_or_create_repo_dir(&repo_dir_str) {
        Ok(repo_path) => {
            if let Err(e) = clone_or_pull_repo(repository_url, "refs/heads/master", &repo_path) {
                print_warning_msg(use_colour, format!("Error: {}\n", e));
            }
        }
        Err(e) => { eprintln!("Failed to create repository directory: {}", e) }
    }
}

pub fn clone_or_pull_repo(repo_url: &str, branch: &str, repo_path: &PathBuf) -> Result<(), Error> {
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
        remote.fetch(&[branch], Some(&mut fetch_options), None)?;

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

pub fn get_or_create_repo_dir(subdir: &str) -> Result<PathBuf, std::io::Error> {
    // Hämta hemkatalog och lägg till underkatalog
    let home_dir = home_dir().expect("Could not determine home directory");
    let repo_dir = home_dir.join(subdir);

    // Skapa katalogen om den inte finns
    if !repo_dir.exists() {
        fs::create_dir_all(&repo_dir.as_path())?;
    }

    Ok(repo_dir)
}
