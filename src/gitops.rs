use std::fs;

use git2::{
    AutotagOption, Error, FetchOptions, ObjectType, RemoteCallbacks, Repository, Cred,
};

use std::path::PathBuf;
use dirs_next::data_dir;

pub fn clone_or_pull_repo(repo_url: &str, branch: &str, repo_path: &PathBuf) -> Result<(), Error> {
    if repo_path.join(".git").exists() {
        println!("Repository exists. Performing 'git pull' on {}...", repo_path.display());
        let repo = Repository::open(repo_path)?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, _username_from_url, _allowed_types| {
            Cred::default()
        });

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        fetch_options.download_tags(AutotagOption::All);

        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&[branch], Some(&mut fetch_options), None)?;

        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

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
            if repo.index()?.has_conflicts() {
                println!("Merge conflicts detected!");
            } else {
                println!("Merge completed without conflicts.");
                repo.checkout_head(Some(&mut git2::build::CheckoutBuilder::default()))?;
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

/// Returns the name of the highest semver tag in the repo, or None if no semver tags exist.
/// Strips a leading `v` before parsing, so both `v1.0.0` and `1.0.0` are recognised.
pub fn latest_semver_tag(repo: &Repository) -> Option<String> {
    let tag_names = repo.tag_names(None).ok()?;
    let mut versions: Vec<(String, semver::Version)> = tag_names
        .iter()
        .flatten()
        .filter_map(|tag| {
            semver::Version::parse(tag.trim_start_matches('v'))
                .ok()
                .map(|v| (tag.to_string(), v))
        })
        .collect();
    versions.sort_by(|a, b| a.1.cmp(&b.1));
    versions.into_iter().last().map(|(tag, _)| tag)
}

/// Checks out a tag by name. Handles both lightweight and annotated tags by
/// peeling to the underlying commit.
pub fn checkout_tag(repo: &Repository, tag_name: &str) -> Result<(), Error> {
    let obj = repo.revparse_single(&format!("refs/tags/{}", tag_name))?;
    let commit_obj = obj.peel(ObjectType::Commit)?;
    repo.checkout_tree(&commit_obj, Some(git2::build::CheckoutBuilder::default().force()))?;
    repo.set_head_detached(commit_obj.id())?;
    Ok(())
}

pub fn get_or_create_repo_dir(subdir: &str) -> Result<PathBuf, std::io::Error> {
    let data_dir = data_dir().expect("Could not determine home directory");
    let repo_dir = data_dir.join(subdir);
    if !repo_dir.exists() {
        fs::create_dir_all(&repo_dir)?;
    }
    Ok(repo_dir)
}
