use std::{ io::{ stdout, Write }, process::exit };
use std::{ fs::{ self, File }, io::{ Cursor }, path::Path };

use online::check;
use crate::{ 
    args::{ GlobalOptions, Color }, 
    color::{ print_success_msg, print_warning_msg },
    settings::get_main_config_file
};
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
use dirs_next::data_dir;

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
            let mut reference = repo.find_reference(branch)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            repo.set_head(branch)?;
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
    let data_dir = data_dir().expect("Could not determine home directory");
    let repo_dir = data_dir.join(subdir);

    // Skapa katalogen om den inte finns
    if !repo_dir.exists() {
        fs::create_dir_all(&repo_dir.as_path())?;
    }

    Ok(repo_dir)
}
