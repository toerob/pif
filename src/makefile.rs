use globwalk::DirEntry;

pub fn add_make_file_entry(makefile: &DirEntry) {
  print!("Add makefile entry to: {:?} ? (y/n, default: y): ", makefile.path());


} 