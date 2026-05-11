use std::{fs, path::Path};

use ansi_term::Colour::*;
use chrono::Local;
use config::Config;
use dialoguer::{Confirm, Input};

use crate::{
    args::{Color, GlobalOptions, InteractiveFictionSystem},
    color::{print_success_msg, print_warning_msg},
    detect::{detect_system, get_extension_path},
    gitops::latest_semver_tag,
    model::{Extension, Extensions, Version},
    settings::get_main_config_file,
    update::update_extensions,
};

// ── Prompt helpers ────────────────────────────────────────────────────────────

fn ask(label: &str, default: Option<&str>) -> String {
    let mut b = Input::<String>::new().with_prompt(label);
    if let Some(d) = default.filter(|d| !d.is_empty()) {
        b = b.default(d.to_string());
    }
    b.interact_text().unwrap_or_default()
}

fn ask_required(label: &str, default: Option<&str>) -> String {
    let mut b = Input::<String>::new()
        .with_prompt(label)
        .validate_with(|s: &String| -> Result<(), &str> {
            if s.is_empty() { Err("Required") } else { Ok(()) }
        });
    if let Some(d) = default.filter(|d| !d.is_empty()) {
        b = b.default(d.to_string());
    }
    b.interact_text().unwrap()
}

fn optional_words(input: &str) -> Option<Vec<String>> {
    (!input.is_empty()).then(|| input.split_whitespace().map(String::from).collect())
}

// ── Misc helpers ──────────────────────────────────────────────────────────────

fn today_iso() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn push_branch(repo: &git2::Repository, branch_name: &str) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, _username, _allowed| git2::Cred::default());
    let mut opts = git2::PushOptions::new();
    opts.remote_callbacks(callbacks);
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[&refspec], Some(&mut opts))
}

fn pr_compare_url(repo_url: &str, branch: &str) -> String {
    format!("{}/compare/{}?expand=1", repo_url.trim_end_matches(".git"), branch)
}

fn load_index_repo_url() -> String {
    let config_file = get_main_config_file().unwrap();
    let settings = Config::builder()
        .add_source(config::File::with_name(config_file.to_str().unwrap()))
        .build()
        .unwrap();
    settings.get_string("main_repository_url").unwrap_or_default()
}

// ── Main ──────────────────────────────────────────────────────────────────────

/// Interactively publish an extension to the pif index.
///
/// Run from inside (or by pointing `dir` at) an extension directory. The function
/// auto-detects as much as it can from the local git repository — remote URL,
/// current branch, latest semver tag, and the global git author — then prompts
/// the user to fill in anything it cannot infer (name, description, version, etc.).
///
/// # Flow
/// 1. **Detection** — resolves `dir`, discovers a git repo if present, reads
///    git config for author and origin URL.
/// 2. **IF system** — detects TADS 3 / Dialog / Inform 6 from the working
///    directory; falls back to an explicit prompt if detection fails.
/// 3. **Prompts** — collects required and optional metadata via [`ask`] /
///    [`ask_required`]. Git entries ask for a branch; archive entries ask for a
///    file extension. `SNAPSHOT` is accepted as a version to track the branch tip.
/// 4. **Merge** — freshens the local index clone via `update_extensions`, then
///    either appends a new [`Version`] to an existing entry or inserts a brand-new
///    [`Extension`].
/// 5. **Preview & confirm** — shows the serialized entry and asks for confirmation
///    before writing.
/// 6. **Commit** — writes the updated YAML, creates a `pif/add-<name>-<version>`
///    branch in the local index repo, stages the index file, and commits.
/// 7. **Push** — attempts to push the branch to `origin`. On success it prints a
///    GitHub compare URL for opening a pull request. On failure it prints manual
///    instructions instead.
pub fn publish_extension(dir: &str, global_options: &GlobalOptions) {
    let use_colours = Color::Never != global_options.color;

    let target_dir = fs::canonicalize(dir)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    // Detect git context from the extension directory
    let local_repo = git2::Repository::discover(&target_dir).ok();

    let default_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let default_url = local_repo.as_ref()
        .and_then(|r| r.find_remote("origin").ok())
        .and_then(|r| r.url().map(String::from));

    let default_branch = local_repo.as_ref()
        .and_then(|r| r.head().ok())
        .and_then(|h| h.shorthand().map(String::from));

    let default_author = git2::Config::open_default()
        .ok()
        .and_then(|c| c.get_string("user.name").ok());

    let default_version = local_repo.as_ref().and_then(|r| latest_semver_tag(r));

    // Detect which IF system we're in to pick the right index file
    let (detected_system, _) = detect_system();
    let system = if detected_system == InteractiveFictionSystem::Unknown {
        println!("Could not auto-detect IF system. Please specify:");
        let s = ask_required("System (tads3 / dialog / inform6)", None);
        match s.to_lowercase().as_str() {
            "tads3"   => InteractiveFictionSystem::Tads3,
            "dialog"  => InteractiveFictionSystem::Dialog,
            "inform6" => InteractiveFictionSystem::Inform6,
            other => {
                eprintln!("Unknown system '{}'. Aborting.", other);
                return;
            }
        }
    } else {
        detected_system
    };

    let index_filename = get_extension_path(system)
        .trim_start_matches("./")
        .to_string();

    println!("\n{}", Yellow.paint("=== pif publish ==="));
    println!("Index file: {}\n", index_filename);

    // ── Prompts ───────────────────────────────────────────────────────────────

    let name     = ask_required("Extension name",        Some(&default_name));
    let author   = ask_required("Author",                default_author.as_deref());
    let desc     = ask_required("Description",           None);
    let homepage = ask("Homepage (optional)",            default_url.as_deref());
    let url      = ask_required("Download or clone URL", default_url.as_deref());

    let is_git = url.ends_with(".git");
    let (branch, ext_field) = if is_git {
        let b = ask_required("Branch", default_branch.as_deref().or(Some("master")));
        (Some(b), None)
    } else {
        let guess = url.rsplit('.').next().map(String::from);
        let e = ask_required(
            "File extension (zip / t / h / ...)",
            guess.as_deref().or(Some("zip")),
        );
        (None, Some(e))
    };

    let default_ver = default_version.as_deref()
        .or(if is_git { Some("SNAPSHOT") } else { None });
    let version = ask_required("Version (e.g. 1.0.0 or SNAPSHOT)", default_ver);

    let type_input = ask_required("Library type (adv3 / adv3lite / both)", Some("adv3"));
    let lib_types: Vec<String> = match type_input.to_lowercase().as_str() {
        "both" => vec!["adv3".into(), "adv3lite".into()],
        t      => vec![t.to_string()],
    };

    let makefile_entries = optional_words(&ask("Makefile entries, space-separated (optional)", None));
    let tags             = optional_words(&ask("Tags, space-separated (optional)",             None));
    let dependencies     = optional_words(&ask("Dependencies, space-separated (optional)",    None));

    let email_input = ask("Your email for PR approval notification (optional)", None);
    let email = (!email_input.is_empty()).then_some(email_input);

    // ── Build Version entry ───────────────────────────────────────────────────

    let parsed_version = {
        let normalized = if version.eq_ignore_ascii_case("SNAPSHOT") {
            "0.0.0-SNAPSHOT".to_string()
        } else if version.matches('.').count() == 1 {
            format!("{}.0", version)
        } else {
            version.clone()
        };
        semver::Version::parse(&normalized).ok()
    };

    let new_version = Version {
        extension_type: Some(lib_types),
        version: parsed_version,
        url: Some(url.clone()),
        makefile_entries,
        ext: ext_field,
        branch,
        last_modified: Some(today_iso()),
    };

    // ── Load & update local master index ─────────────────────────────────────

    update_extensions(global_options);

    let repo_dir = dirs_next::data_dir()
        .expect("Could not determine data directory")
        .join("pif")
        .join("repo");

    let index_path = repo_dir.join(&index_filename);

    let existing_yaml = fs::read_to_string(&index_path)
        .unwrap_or_else(|_| "schema-version: 1\n\nextensions:\n".to_string());

    let mut data: Extensions = serde_yaml::from_str(&existing_yaml).unwrap_or(Extensions {
        schema_version: Some(1),
        extensions: Vec::new(),
    });

    let is_update = data.extensions.iter()
        .any(|e| e.name.to_lowercase() == name.to_lowercase());

    if is_update {
        let existing = data.extensions.iter_mut()
            .find(|e| e.name.to_lowercase() == name.to_lowercase())
            .unwrap();
        existing.versions.push(new_version);
        println!("\n{}", Yellow.paint(format!("Adding new version to existing '{}' entry.", name)));
    } else {
        data.extensions.push(Extension {
            name: name.clone(),
            author: Some(author.clone()),
            desc: Some(desc.clone()),
            homepage: (!homepage.is_empty()).then_some(homepage.clone()),
            tags,
            dependencies,
            versions: vec![new_version],
        });
        println!("\n{}", Yellow.paint(format!("Creating new '{}' entry.", name)));
    }

    let preview = data.extensions.iter()
        .find(|e| e.name.to_lowercase() == name.to_lowercase())
        .map(|e| serde_yaml::to_string(e).unwrap_or_default())
        .unwrap_or_default();
    println!("\n{}\n{}", Yellow.paint("Entry preview:"), preview);

    if !Confirm::new()
        .with_prompt("Submit this as a PR to the pif index?")
        .default(true)
        .interact()
        .unwrap()
    {
        println!("Aborted.");
        return;
    }

    let new_yaml = serde_yaml::to_string(&data).unwrap();
    fs::write(&index_path, &new_yaml).expect("Failed to write index file");

    // ── Git: commit on feature branch ────────────────────────────────────────

    let index_repo = git2::Repository::open(&repo_dir)
        .expect("Could not open local index repo");

    let branch_name = format!(
        "pif/add-{}-{}",
        name.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric() && c != '-', "-"),
        version.to_lowercase()
    );

    let head_commit = {
        let head = index_repo.head().unwrap();
        index_repo.find_commit(head.target().unwrap()).unwrap()
    };

    let _ = index_repo.branch(&branch_name, &head_commit, false);

    let mut git_index = index_repo.index().unwrap();
    git_index.add_path(Path::new(&index_filename)).unwrap();
    git_index.write().unwrap();
    let tree_oid = git_index.write_tree().unwrap();
    let tree = index_repo.find_tree(tree_oid).unwrap();

    let sig = index_repo.signature().unwrap_or_else(|_| {
        let email_str = email.as_deref().unwrap_or("pif@localhost");
        git2::Signature::now(&author, email_str).unwrap()
    });

    let pr_body = {
        let mut body = format!("Adds **{}** `{}` to the pif index.\n", name, version);
        if let Some(ref e) = email {
            body += &format!("\nContact for approval notification: {}", e);
        }
        body
    };

    index_repo.commit(
        Some(&format!("refs/heads/{}", branch_name)),
        &sig, &sig,
        &format!("Add {} {}\n\n{}", name, version, pr_body),
        &tree,
        &[&head_commit],
    ).expect("Failed to create commit");

    let branch_ref = index_repo
        .revparse_single(&format!("refs/heads/{}", branch_name))
        .unwrap();
    index_repo
        .checkout_tree(&branch_ref, Some(git2::build::CheckoutBuilder::default().force()))
        .unwrap();
    index_repo
        .set_head(&format!("refs/heads/{}", branch_name))
        .unwrap();

    // ── Push & report ────────────────────────────────────────────────────────

    let repo_url = load_index_repo_url();

    match push_branch(&index_repo, &branch_name) {
        Ok(_) => {
            let link = pr_compare_url(&repo_url, &branch_name);
            print_success_msg(use_colours, "\nBranch pushed successfully!\n".to_string());
            println!("Open your pull request here:");
            println!("  {}\n", Green.paint(&link));
        }
        Err(e) => {
            print_warning_msg(use_colours, format!("\nCould not push to remote ({})\n", e));
            println!("The commit is ready locally on branch '{}'.", branch_name);
            println!("To submit manually:");
            println!("  1. Fork {}", repo_url);
            println!("  2. Add it as a remote and push branch '{}'", branch_name);
            println!("  3. Open a pull request\n");
        }
    }
}
