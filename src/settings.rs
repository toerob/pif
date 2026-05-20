use dirs_next::config_dir;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::Deserialize;
use serde_yaml::{Mapping, Value};

#[derive(Deserialize, Default)]
pub struct PifConfig {
    #[serde(default)]
    pub install_dirs: HashMap<String, String>,
    pub verbose_level: Option<usize>,
}

lazy_static::lazy_static! {
    pub static ref VERBOSE_DEFAULT: usize = load_config().verbose_level.unwrap_or(2);
}

pub fn load_config() -> PifConfig {
    match get_main_config_file() {
        Ok(path) => fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_yaml::from_str(&s).ok())
            .unwrap_or_default(),
        Err(_) => PifConfig::default(),
    }
}

/// Expand a leading `~/` to the user's home directory.
pub fn expand_path(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs_next::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(s)
}

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
r#"main_repository_url: https://github.com/toerob/if-extensions
main_repository_branch: main

# Override the default installation directory per system.
# If omitted, inform uses the platform Extensions folder; all others use ".".
# install_dirs:
#   tads3: ~/path/to/tads3/extensions
#   dialog: ~/path/to/dialog/extensions
#   inform: ~/path/to/inform/extensions
#   inform6: ~/path/to/inform6/extensions
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





pub fn reset_install_dir(system: &str) -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let config_path = get_main_config_file()?;
    let content = fs::read_to_string(&config_path)?;

    let mut root: Value = serde_yaml::from_str(&content)
        .unwrap_or(Value::Mapping(Mapping::new()));

    let removed = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(&Value::String("install_dirs".into())))
        .and_then(|v| v.as_mapping_mut())
        .map(|dirs| dirs.remove(&Value::String(system.into())).is_some())
        .unwrap_or(false);

    fs::write(&config_path, serde_yaml::to_string(&root)?)?;
    Ok((config_path, removed))
}

pub fn set_verbose_level(level: usize) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if !(1..=3).contains(&level) {
        return Err(format!("Invalid verbose level '{}': must be 1, 2, or 3", level).into());
    }
    let config_path = get_main_config_file()?;
    let content = fs::read_to_string(&config_path)?;

    let mut root: Value = serde_yaml::from_str(&content)
        .unwrap_or(Value::Mapping(Mapping::new()));

    root.as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?
        .insert(Value::String("verbose_level".into()), Value::Number(level.into()));

    fs::write(&config_path, serde_yaml::to_string(&root)?)?;
    Ok(config_path)
}

pub fn reset_verbose_level() -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let config_path = get_main_config_file()?;
    let content = fs::read_to_string(&config_path)?;

    let mut root: Value = serde_yaml::from_str(&content)
        .unwrap_or(Value::Mapping(Mapping::new()));

    let removed = root
        .as_mapping_mut()
        .map(|m| m.remove(&Value::String("verbose_level".into())).is_some())
        .unwrap_or(false);

    fs::write(&config_path, serde_yaml::to_string(&root)?)?;
    Ok((config_path, removed))
}

pub fn reset_all_install_dirs() -> Result<(PathBuf, usize), Box<dyn std::error::Error>> {
    let config_path = get_main_config_file()?;
    let content = fs::read_to_string(&config_path)?;

    let mut root: Value = serde_yaml::from_str(&content)
        .unwrap_or(Value::Mapping(Mapping::new()));

    let count = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(&Value::String("install_dirs".into())))
        .and_then(|v| v.as_mapping_mut())
        .map(|dirs| { let n = dirs.len(); dirs.clear(); n })
        .unwrap_or(0);

    fs::write(&config_path, serde_yaml::to_string(&root)?)?;
    Ok((config_path, count))
}

pub fn set_install_dir(system: &str, directory: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let expanded = expand_path(directory);
    fs::create_dir_all(&expanded)
        .map_err(|e| format!("Invalid directory '{}': {}", expanded.display(), e))?;

    let config_path = get_main_config_file()?;
    let content = fs::read_to_string(&config_path)?;

    let mut root: Value = serde_yaml::from_str(&content)
        .unwrap_or(Value::Mapping(Mapping::new()));

    let mapping = root.as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?;

    let install_dirs_key = Value::String("install_dirs".into());
    let dirs = mapping
        .entry(install_dirs_key)
        .or_insert(Value::Mapping(Mapping::new()));

    dirs.as_mapping_mut()
        .ok_or("install_dirs is not a YAML mapping")?
        .insert(Value::String(system.into()), Value::String(directory.into()));

    fs::write(&config_path, serde_yaml::to_string(&root)?)?;
    Ok(config_path)
}

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
