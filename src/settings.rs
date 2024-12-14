use dirs_next::{ config_dir };
use std::fs;
use std::path::PathBuf;
use std::io::{ self, Write, Read };

pub fn get_main_config_file() -> Result<PathBuf, io::Error> {
    let config_dir = config_dir()
        .expect("Could not determine config directory")
        .join("ifp")
        .join("config");

    if !config_dir.exists() {
        println!("Missing configuration directory, creating it. ");
        fs::create_dir_all(&config_dir)?;
    }

    let config_file = config_dir.join("config.yaml");

    // Create a default file if none exists
    if !config_file.exists() {
        println!("No config file exists. Writing a default one");
        let default_content =
r#"main_repository_url: https://github.com/toerob/t3cartographer
main_repository_branch: master
"#;
        fs::write(&config_file, default_content)?;
        println!("Default settings written to {:?}", config_file);
    }

    Ok(config_file)
}

pub fn write_default_settings() -> Result<(), io::Error> {
    let config_file = get_main_config_file()?;
    println!("*** Checking config file ***");

    if !config_file.exists() {
        println!("No config file exists. Writing a default one");
        let default_content =
            r#"main_repository_url: https://github.com/toerob/t3cartographer
main_repository_branch: master
"#;
        fs::write(&config_file, default_content)?;
        println!("Default settings written to {:?}", config_file);
    }
    Ok(())
}
