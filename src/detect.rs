use globwalk::{glob, DirEntry};

use crate::args::InteractiveFictionSystem;

pub fn get_extension_path(system_type: InteractiveFictionSystem) -> Option<String> {
    match system_type {
        InteractiveFictionSystem::Tads3  => Some("./tads3-extensions.yaml".to_owned()),
        InteractiveFictionSystem::Inform6 => Some("./inform6-extensions.yaml".to_owned()),
        InteractiveFictionSystem::Dialog  => Some("./dialog-extensions.yaml".to_owned()),
        _ => None,
    }
}


/**
 * Returns the system detected and an optional makefile if that is also detected
 */
pub fn detect_system() -> (InteractiveFictionSystem, Option<DirEntry>) {
    //let current_directory = env::current_dir().unwrap();
    //return (InteractiveFictionSystem::Dialog, None); // TODO: add override via function

    let shortest_path = glob("*.{t3m}").unwrap()
        .filter_map(Result::ok) 
        .min_by_key(|e| e.path().display().to_string().len());
        
    if let Some(_file) = &shortest_path {
        //println!("Makefile shortest path: {}\n", &file.path().display().to_string());
        return (InteractiveFictionSystem::Tads3, shortest_path)
    }    

    for entry in glob("*.{t3m}").expect("Failed to read tads3 glob pattern") {
        
        match entry {
            Ok(path) => return (InteractiveFictionSystem::Tads3, Some(path)),
            Err(_) => {
                eprintln!("***ERROR**");
            }
        }
    }
    for entry in glob("*.{inf}").expect("Failed to read inform6 glob pattern") {
        match entry {
            Ok(_) => return (InteractiveFictionSystem::Inform6, None),
            Err(_) => {
                eprintln!("***ERROR**");
            }
        }
    }
    for entry in glob("*.{dg}").expect("Failed to read dialog glob pattern") {
        match entry {
            Ok(_) => return (InteractiveFictionSystem::Dialog, None),
            Err(_) => {
                eprintln!("***ERROR**");
            }
        }
    }
    return (InteractiveFictionSystem::Unknown, None);
}


#[cfg(test)]
mod tests {
    

    #[test]
    fn get_extension_path_works() {
    }
}
