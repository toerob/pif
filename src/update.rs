use std::{io::{stdout, Write}, process::exit};

use online::check;
use crate::{args::{GlobalOptions, Color}, color::{colorize_message, print_success_msg}};
use git2::{ErrorCode, Repository};



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
        
        print_success_msg(use_colour, format!(" ==> Downloading latest changes from: {} \n", repository_url));
        //println!("{}", colorize_message(success_color,format!(" ==> Downloading latest changes from: {} ", repository_url)));
        stdout().flush().unwrap();
        Repository::clone(repository_url, workspace_folder)
            .expect("failed to clone repository: {}");
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

fn pull_latest(global_options: &GlobalOptions, workspace_folder: &str, remote_branch: &String) -> () {
    let use_colour = Color::Never != global_options.color;
    
    /*let success_color = match Color::Never != global_options.color {
        true => Some(Green),
        false => None,
    };

    //print!("{}", colorize_message(success_color, String::from(" ==> Fetching latest changes")));*/

    print_success_msg(use_colour, String::from(" ==> Fetching latest changes\n"));


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
