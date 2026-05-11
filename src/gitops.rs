use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use git2::{Error, ObjectType, Repository};
use dirs_next::data_dir;

pub fn clone_or_pull_repo(repo_url: &str, branch: &str, repo_path: &PathBuf) -> Result<(), String> {
    let run = |args: &[&str]| -> Result<(), String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    };

    if repo_path.join(".git").exists() {
        println!("Repository exists. Updating {}...", repo_path.display());
        run(&["fetch", "--tags", "origin", branch])?;
        run(&["reset", "--hard", "FETCH_HEAD"])?;
        println!("Updated successfully.");
    } else {
        println!("Cloning repository from {} to {:?}", repo_url, repo_path);
        run(&["clone", repo_url, "."])?;
        println!("Repository cloned successfully.");
    }
    Ok(())
}

/// Returns the name of the highest semver tag in the repo, or None if no semver tags exist.
/// Strips a leading `v` before parsing, so both `v1.0.0` and `1.0.0` are recognised.
pub fn latest_semver_tag(repo_path: &Path) -> Option<String> {
    let repo = Repository::open(repo_path).ok()?;
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
pub fn checkout_tag(repo_path: &Path, tag_name: &str) -> Result<(), Error> {
    let repo = Repository::open(repo_path)?;
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
