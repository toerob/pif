use std::path::PathBuf;
use serde_yaml::Value;

use super::{load_yaml, save_yaml};

pub fn set_verbose_level(level: usize) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if !(1..=3).contains(&level) {
        return Err(format!("Invalid verbose level '{}': must be 1, 2, or 3", level).into());
    }

    let (path, mut root) = load_yaml()?;

    root.as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?
        .insert(Value::String("verbose_level".into()), Value::Number(level.into()));

    save_yaml(&path, &root)?;
    Ok(path)
}

pub fn reset_verbose_level() -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let removed = root
        .as_mapping_mut()
        .map(|m| m.remove(&Value::String("verbose_level".into())).is_some())
        .unwrap_or(false);

    save_yaml(&path, &root)?;
    Ok((path, removed))
}
