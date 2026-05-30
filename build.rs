// build.rs — Cargo build script.
//
// Cargo automatically compiles and runs this file before building the crate,
// purely because it is named "build.rs" and lives at the crate root. Nothing
// in the main source calls it. The output (cargo:rerun-if-changed directives,
// generated files, etc.) is produced before rustc sees any of src/.

use std::fs;
use std::path::Path;
use clap::CommandFactory;

// Stub for `crate::config::VERBOSE_DEFAULT` referenced in args.rs.
// build.rs is a separate crate, so `crate::config` here refers to this module,
// not the real config module. The lazy_static type must match: a Deref to usize.
mod config {
    lazy_static::lazy_static! {
        pub static ref VERBOSE_DEFAULT: usize = 2;
    }
}

include!("src/args.rs");

fn main() {
    // Re-run only when the CLI definition changes.
    println!("cargo:rerun-if-changed=src/args.rs");

    let out_dir = Path::new("assets/man");
    fs::create_dir_all(out_dir).unwrap();

    let cmd = InteractiveFictionToolArgs::command();
    // Start recursion with the top-level name "pif".
    generate_man_pages(cmd, "pif".to_string(), out_dir);
}

/// Recursively generates a `<full-name>.1` file for every (sub)command,
/// naming each page with the fully-qualified hyphenated path so that
/// sub-pages don't clobber system-wide man pages (e.g. `pif-config-dir.1`
/// instead of the ambiguous `dir.1`).
///
/// Files are only written when their content has changed, keeping git diffs
/// clean on rebuilds that touch unrelated source files.
fn generate_man_pages(cmd: clap::Command, full_name: String, out_dir: &Path) {
    // Recurse into sub-commands first so a parent page can reference them.
    for sub in cmd.get_subcommands().filter(|s| !s.is_hide_set()).cloned() {
        let sub_full = format!("{}-{}", full_name, sub.get_name());
        generate_man_pages(sub, sub_full, out_dir);
    }

    // Rename the command to its full path so the filename and page title
    // both use the qualified form (e.g. "pif-config-dir").
    // Command::name requires &'static str, so we leak the String — acceptable
    // in a build script where the process exits when done.
    let static_name: &'static str = Box::leak(full_name.into_boxed_str());
    let named_cmd = cmd.name(static_name);
    let man = clap_mangen::Man::new(named_cmd);
    let path = out_dir.join(man.get_filename());
    let mut buf = Vec::new();
    man.render(&mut buf).unwrap();
    let existing = fs::read(&path).unwrap_or_default();
    if existing != buf {
        fs::write(&path, &buf).unwrap();
    }
}
