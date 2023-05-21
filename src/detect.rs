use globwalk::{glob, DirEntry};

#[derive(Debug)]
pub enum InteractiveFictionSystem {
    Tads3,
    Dialog,
    Inform6,
    Inform7,
    Unknown,
}


pub fn get_extension_path(system_type: InteractiveFictionSystem) -> String {
  match system_type {
      InteractiveFictionSystem::Tads3 => "./tads3-extensions.json",
      InteractiveFictionSystem::Inform6 => "./inform6-extensions.json",
      InteractiveFictionSystem::Dialog => "./dialog-extensions.json",
      _ => panic!("System not supported yet"),
  }
  .to_owned()
}


/**
 * Returns the system detected and an optional makefile if that is also detected
 */
pub fn detect_system() -> (InteractiveFictionSystem, Option<DirEntry>) {
    //let current_directory = env::current_dir().unwrap();
    //return (InteractiveFictionSystem::Dialog, None); // TODO: add override via function

    for entry in glob("*.{t3m}").expect("Failed to read tads3 glob pattern") {
        match entry {
            Ok(path) => return (InteractiveFictionSystem::Tads3, Some(path)),
            Err(e) => {
                println!("***ERROR**");
            }
        }
    }
    for entry in glob("*.{inf}").expect("Failed to read inform6 glob pattern") {
        match entry {
            Ok(_) => return (InteractiveFictionSystem::Inform6, None),
            Err(e) => {
                println!("***ERROR**");
            }
        }
    }
    for entry in glob("*.{dg}").expect("Failed to read dialog glob pattern") {
        match entry {
            Ok(_) => return (InteractiveFictionSystem::Dialog, None),
            Err(e) => {
                println!("***ERROR**");
            }
        }
    }
    return (InteractiveFictionSystem::Unknown, None);
}
