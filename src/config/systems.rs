use std::path::PathBuf;
use serde_yaml::Value;

use super::{load_yaml, save_yaml};

pub fn set_systems(systems: &[String]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let seq = Value::Sequence(systems.iter().map(|s| Value::String(s.clone())).collect());

    root.as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?
        .insert(Value::String("systems".into()), seq);

    save_yaml(&path, &root)?;
    Ok(path)
}

pub fn add_systems(systems: &[String]) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let seq = root
        .as_mapping_mut()
        .ok_or("config root is not a YAML mapping")?
        .entry(Value::String("systems".into()))
        .or_insert(Value::Sequence(vec![]));

    let seq = seq.as_sequence_mut()
        .ok_or("systems is not a YAML sequence")?;

    for sys in systems {
        let v = Value::String(sys.clone());
        if !seq.contains(&v) {
            seq.push(v);
        }
    }

    save_yaml(&path, &root)?;
    Ok(path)
}

pub fn remove_systems(systems: &[String]) -> Result<(PathBuf, Vec<String>), Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let mut removed = Vec::new();

    if let Some(seq) = root
        .as_mapping_mut()
        .and_then(|m| m.get_mut(&Value::String("systems".into())))
        .and_then(|v| v.as_sequence_mut())
    {
        for sys in systems {
            let v = Value::String(sys.clone());
            if let Some(pos) = seq.iter().position(|x| x == &v) {
                seq.remove(pos);
                removed.push(sys.clone());
            }
        }
    }

    save_yaml(&path, &root)?;
    Ok((path, removed))
}

pub fn reset_systems() -> Result<(PathBuf, bool), Box<dyn std::error::Error>> {
    let (path, mut root) = load_yaml()?;

    let removed = root
        .as_mapping_mut()
        .map(|m| m.remove(&Value::String("systems".into())).is_some())
        .unwrap_or(false);

    save_yaml(&path, &root)?;
    Ok((path, removed))
}
