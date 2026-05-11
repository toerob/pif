use dirs_next::{ config_dir };
use std::fs;
use std::path::PathBuf;
use std::io::{ self };

pub fn get_main_config_file() -> Result<PathBuf, io::Error> {
    let config_dir = config_dir()
        .expect("Could not determine config directory")
        .join("pif")
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



/*
pub fn get_main_repository_local_path_and_branch() -> (&str, &str) {

  println!("GONG!!");
  let config_file_pathbuf = get_main_config_file().unwrap();
  let config_file_path_str = config_file_pathbuf.to_str().unwrap();
  
  println!("***{}***", config_file_path_str);

  let settings = Config::builder()
      .add_source(config::File::with_name(config_file_path_str))
      .build()
      .unwrap();

  //let config = settings.clone().try_deserialize::<HashMap<String, String>>().unwrap();

  let repository_url = settings.get_string("main_repository_url").unwrap().clone();
  let repository_main_branch = settings.get_string("main_repository_branch").unwrap().clone();


  println!("{:?}", &repository_url);
  println!("{:?}", &repository_main_branch);

  //(repository.to_string(), repository_main_branch.to_string());
  ("sadf", "ssdfsdf")
}*/

/*
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
}*/





#[cfg(test)]
mod tests {
    
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_get_main_config_file() {
        /*let temp_dir = std::env::temp_dir().join("test_prepare_environment");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }

        get_main_config_file(&temp_dir).unwrap();
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());

        fs::remove_dir_all(&temp_dir).unwrap();
        */
    }
}
