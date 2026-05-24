use std::path::PathBuf;
use serde_yaml::Value;

use super::{expand_path, load_yaml, save_yaml};

pub fn set_install_dir(system: &str, directory: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let expanded = expand_path(directory);
    std::fs::create_dir_all(&expanded)
        .map_err(|e| format!("Invalid directory '{}': {}", expanded.display(), e))?;

    let (path, mut root) = load_yaml()?;

    let mapping = root.as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?;

    let dirs = mapping
        .entry(Value::String("install_dirs".into()))
        .or_insert(Value::Mapping(serde_yaml::Mapping::new()));

    dirs.as_mapping_mut()
        .ok_or("install_dirs is not a YAML mapping")?
        .insert(Value::String(system.into()), Value::String(directory.into()));

    save_yaml(&path, &root)?;
    Ok(path)
}

pub fn reset_install_dir(system: &str) -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let removed = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(&Value::String("install_dirs".into())))
        .and_then(|v| v.as_mapping_mut())
        .map(|dirs| dirs.remove(&Value::String(system.into())).is_some())
        .unwrap_or(false);

    save_yaml(&path, &root)?;
    Ok((path, removed))
}

pub fn reset_all_install_dirs() -> Result<(PathBuf, usize), Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let count = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(&Value::String("install_dirs".into())))
        .and_then(|v| v.as_mapping_mut())
        .map(|dirs| { let n = dirs.len(); dirs.clear(); n })
        .unwrap_or(0);

    save_yaml(&path, &root)?;
    Ok((path, count))
}
