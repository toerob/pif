use crate::args::GlobalOptions;
use git2::{ErrorCode, Repository};
use std::{fs, path::Path};

pub fn update_extensions(_: &GlobalOptions) {
    let workspace_folder = ".ifp";
    let repository_url = "https://github.com/toerob/t3cartographer";

    let repository_main_branch = "master"; // TODO: rename to main when the public repository exists
    if !Path::new(workspace_folder).exists() {
        print!(" ==> Creating workspace directory: {} ", workspace_folder);
        fs::create_dir(workspace_folder).expect("Could not create .ifp folder");
        println!(" Done!");
    }

    if !ensure_open(workspace_folder) {
        print!(" ==> Downloading latest changes from: {} ", repository_url);
        Repository::clone(repository_url, workspace_folder)
            .expect("failed to clone repository: {}");
        println!(" Done!");
    }
    pull_latest(workspace_folder, &repository_main_branch.to_string());
}

fn ensure_open(workspace_folder: &str) -> bool {
    //println!(" ==> Ensuring {} exists", &workspace_folder);
    match Repository::open(workspace_folder) {
        Ok(_) => true,
        Err(e) => {
            if ErrorCode::NotFound != e.code() {
                panic!("failed to open: {}", e); // If due to other error than NotFound, let's panic!
            }
            false
        }
    }
}

fn pull_latest(workspace_folder: &str, remote_branch: &String) -> () {
    print!(" ==> Fetching latest changes");

    let origin_name = "origin";
    let repo = Repository::open(workspace_folder).expect("Failed to open repository");
    let mut origin = repo.find_remote(origin_name).expect("Find origin failed");
    origin
        .fetch(&[remote_branch], None, None)
        .expect("Fetch origin failed");
    let fetch_head = repo.find_reference("FETCH_HEAD").expect("msg");
    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .expect("...");
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
}
