use std::path::PathBuf;
use serde_yaml::Value;

use super::{load_yaml, save_yaml};

pub fn set_system_versions(system: &str, versions: &[String]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let sys_versions = root
        .as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?
        .entry(Value::String("system_versions".into()))
        .or_insert(Value::Mapping(serde_yaml::Mapping::new()));

    let seq = Value::Sequence(versions.iter().map(|v| Value::String(v.clone())).collect());

    sys_versions
        .as_mapping_mut()
        .ok_or("system_versions is not a YAML mapping")?
        .insert(Value::String(system.into()), seq);

    save_yaml(&path, &root)?;
    Ok(path)
}

pub fn add_system_version_specs(system: &str, versions: &[String]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let sys_versions = root
        .as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?
        .entry(Value::String("system_versions".into()))
        .or_insert(Value::Mapping(serde_yaml::Mapping::new()));

    let seq = sys_versions
        .as_mapping_mut()
        .ok_or("system_versions is not a YAML mapping")?
        .entry(Value::String(system.into()))
        .or_insert(Value::Sequence(vec![]));

    let seq = seq.as_sequence_mut()
        .ok_or("system version list is not a YAML sequence")?;

    for ver in versions {
        let v = Value::String(ver.clone());
        if !seq.contains(&v) {
            seq.push(v);
        }
    }

    save_yaml(&path, &root)?;
    Ok(path)
}

pub fn remove_system_version_specs(system: &str, versions: &[String]) -> Result<(PathBuf, Vec<String>), Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let mut removed = Vec::new();

    if let Some(seq) = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(&Value::String("system_versions".into())))
        .and_then(|v| v.as_mapping_mut())
        .and_then(|m| m.get_mut(&Value::String(system.into())))
        .and_then(|v| v.as_sequence_mut())
    {
        for ver in versions {
            let v = Value::String(ver.clone());
            if let Some(pos) = seq.iter().position(|x| x == &v) {
                seq.remove(pos);
                removed.push(ver.clone());
            }
        }
    }

    save_yaml(&path, &root)?;
    Ok((path, removed))
}

pub fn reset_system_versions(system: Option<&str>) -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let removed = match system {
        Some(sys) => root
            .as_mapping_mut()
            .and_then(|m| m.get_mut(&Value::String("system_versions".into())))
            .and_then(|v| v.as_mapping_mut())
            .map(|m| m.remove(&Value::String(sys.into())).is_some())
            .unwrap_or(false),
        None => root
            .as_mapping_mut()
            .map(|m| m.remove(&Value::String("system_versions".into())).is_some())
            .unwrap_or(false),
    };

    save_yaml(&path, &root)?;
    Ok((path, removed))
}
