use clap::{Args, Parser, Subcommand, ValueEnum};


#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = "\
pif is a package manager for interactive fiction extension libraries.

It maintains a local copy of the IF extensions registry (refreshed with \
`pif update`) and lets you browse, install, and manage libraries for TADS 3, \
Inform 7, Inform 6, Dialog, Hugo, ZIL, and other IF authoring systems.

WORKFLOW

  1. pif update          — download the latest registry index
  2. pif list            — browse available extensions
  3. pif install <name>  — install an extension into the system's directory
  4. pif info <name>     — view full metadata and version history

SYSTEMS

Extensions are tagged with the IF system they target. pif auto-detects the \
system from files in the current directory:

  Makefile / .t3m   → tads3
  .inf              → inform6
  .i7x / .materials → inform
  .dg               → dialog
  .hug              → hugo

Override detection with --system, or use --system=all to work across every \
known system at once.

CONFIGURATION

Per-system install directories, verbosity, and version constraints are stored \
in the pif config file:

  Linux/macOS  $XDG_CONFIG_HOME/pif/config.yaml  (usually ~/.config/pif/)
  Windows      %APPDATA%\\pif\\config.yaml

Manage the config with the `pif config` subcommand.")]
pub struct InteractiveFictionToolArgs {
    #[clap(subcommand)]
    pub menu: MenuSubCommand,

    #[clap(flatten)]
    pub global_options: GlobalOptions,
}

#[derive(Debug, Subcommand)]
pub enum MenuSubCommand {
    /// Show information about an extension
    ///
    /// Displays the full metadata record for one or more extensions: name,
    /// description, author, supported IF system, available versions, homepage,
    /// and associated tags.
    ///
    /// When called without a name the filter flags (--author, --keyword, --tag)
    /// narrow the set of all known extensions. Combine filters freely; results
    /// must satisfy every filter given.
    Info(InfoCommand),

    /// Fetch the latest extension registry from upstream
    ///
    /// Downloads the master index from the pif registry repository and stores
    /// it in the local application data directory. Run this periodically to
    /// pick up newly published extensions and updated version lists.
    Update(UpdateCommand),

    /// Install one or more extensions
    ///
    /// Downloads and installs the requested extensions into the configured
    /// system-specific directory. Specify a version with a colon suffix
    /// (e.g. t3cartographer:1.0); omit the version to install LATEST.
    ///
    /// Version strings are prefix-matched against the registry's version list,
    /// so "1" matches "1.0", "1.1", etc. — the latest matching entry is used.
    ///
    /// The target directory defaults to the path configured for the active
    /// system (see `pif config dir`). Override it for a single run with
    /// --directory.
    Install(InstallCommand),

    /// List available extensions
    ///
    /// Prints all extensions in the local registry, optionally filtered and
    /// sorted. Results are limited to the active --system unless --system=all
    /// is given.
    ///
    /// Use --author and --keyword for fuzzy substring matching; use --tag for
    /// an exact tag match. Change output format with --presentation.
    List(ListCommand),

    /// Publish an extension to the pif index
    ///
    /// Prepares a contribution for the upstream registry. Reads release
    /// metadata from release.yaml in DIRECTORY (defaults to the current
    /// directory), validates it against the registry schema, and opens the
    /// draft pull-request workflow.
    ///
    /// You need a fork of the registry repository and GitHub credentials
    /// configured before running this command.
    Publish(PublishCommand),

    /// List all available tags
    ///
    /// Prints every tag that appears across the registry. Tags are short
    /// category labels (e.g. "library", "parser", "graphics") attached to
    /// individual extensions.
    ///
    /// Use a tag as a filter with --tag in the list, search, or info commands
    /// to narrow results to a specific category.
    Tags(TagsCommand),

    /// Search extensions by name and description
    ///
    /// Performs a case-insensitive substring match of QUERY against the
    /// extension name and description fields. Combine with --author,
    /// --keyword, or --tag to narrow results further.
    Search(SearchCommand),

    /// Manage pif configuration
    ///
    /// Reads and writes the pif configuration file. The file controls
    /// per-system install directories, the default verbosity level, the
    /// system filter used in Auto mode, and version constraints.
    ///
    /// Config file location:
    ///
    ///   Linux/macOS  $XDG_CONFIG_HOME/pif/config.yaml
    ///   Windows      %APPDATA%\pif\config.yaml
    ///
    /// Use `pif config show` to view the current file with syntax highlighting.
    Config(ConfigCommand),

    /// Manage the local extension registry
    ///
    /// pif records every extension it installs (name, version, install path).
    /// Use these subcommands to inspect or correct that record — for example
    /// after moving files manually or to force a clean reinstall.
    Registry(RegistryCommand),
}

#[derive(Debug, Args)]
pub struct UpdateCommand {
}

#[derive(Debug, Args)]
pub struct PublishCommand {
    /// Path to the extension directory (default: current directory)
    ///
    /// The directory must contain a valid release.yaml describing the
    /// extension. pif validates the file against the registry schema before
    /// opening the contribution workflow.
    #[arg(default_value = ".")]
    pub directory: String,
}


#[derive(Debug, Args)]
pub struct InstallCommand {
    /// Extensions to install, each optionally pinned to a version
    ///
    /// Each argument is a bare extension name or name:version.
    ///
    /// Examples:
    ///
    ///   pif install t3cartographer           # install LATEST
    ///   pif install t3cartographer:1.0       # install exactly 1.0
    ///   pif install conspace:2.1 t3cart:1.0  # multiple at once
    ///
    /// Version strings are prefix-matched, so "1" matches "1.0", "1.1", etc.
    /// When multiple versions match, the latest is chosen.
    pub names: Vec<String>,

    #[clap(flatten)]
    pub install_options: InstallOptions,
}

#[derive(Debug, Args)]
pub struct InfoCommand {
    /// Extension names to look up (omit to match all extensions)
    ///
    /// When no names are given, the filter flags (--author, --keyword, --tag)
    /// determine which extensions are shown. Leaving both names and filters
    /// empty prints info for every extension in the registry.
    pub name: Vec<String>,

    /// Filter by author (fuzzy search)
    ///
    /// Case-insensitive substring match against the author field. Can be
    /// combined with --keyword and --tag.
    #[clap(long, short, global = false)]
    pub author: Option<String>,

    /// Filter by keyword (fuzzy search)
    ///
    /// Case-insensitive substring match against the extension description and
    /// keyword fields.
    #[clap(long, short, global = false)]
    pub keyword: Option<String>,

    /// Filter by tag (exact match)
    ///
    /// Only extensions that carry this exact tag are shown. Use `pif tags` to
    /// see all available tag names.
    #[clap(long, short = 't', global = false)]
    pub tag: Option<String>,
}


#[derive(Debug, Subcommand)]
pub enum ListSubCommand {
    List(ListCommand),
}

#[derive(Debug, Args)]
pub struct ListCommand {
    #[clap(flatten)]
    pub list_options: ListOptions,
}

#[derive(Debug, Args)]
pub struct SearchCommand {
    /// Text to search for in extension names and descriptions
    ///
    /// The search is case-insensitive and matches any substring. Combine with
    /// --author, --keyword, or --tag for more targeted results, or use
    /// `pif list` when you want to browse without a search term.
    pub query: String,

    #[clap(flatten)]
    pub list_options: ListOptions,
}

#[derive(Debug, Args)]
pub struct InstallOptions {
    /// Override the install directory for this invocation
    ///
    /// By default, extensions are installed to the directory configured for
    /// the target system (see `pif config dir`). If no directory is configured
    /// for a system, pif will prompt on first install.
    ///
    /// Common defaults:
    ///
    ///   tads3    ~/tads3/extensions/  (or as set in your TADS3 Makefile)
    ///   inform   ~/Inform/Extensions/
    ///   inform6  ~/inform/lib/
    ///   dialog   ~/dialog/lib/
    ///
    /// Paths starting with ~/ are expanded to your home directory.
    #[arg(short = 'd', long = "directory", value_name = "FOLDER")]
    pub installation_directory: Option<String>,
}


#[derive(Debug, Args)]
pub struct ListOptions {
    /// Filter by author (fuzzy search)
    ///
    /// Case-insensitive substring match against the author field.
    #[clap(long, short, global = false)]
    pub author: Option<String>,

    /// Filter by keyword (fuzzy search)
    ///
    /// Case-insensitive substring match against the description and keyword
    /// fields.
    #[clap(long, short, global = false)]
    pub keyword: Option<String>,

    /// Filter by tag (exact match)
    ///
    /// Only extensions carrying this exact tag are included. Use `pif tags`
    /// to see all available tag names.
    #[clap(long, short = 't', global = false)]
    pub tag: Option<String>,

    /// Result ordering direction (ascending or descending)
    #[clap(long, short = 'o', value_enum, global = true, default_value_t = OrderingDirection::Descending)]
    pub ordering_direction: OrderingDirection,

    /// Property to sort results by
    #[clap(long, short = 'p', value_enum, global = true, default_value_t = SortProperty::Name)]
    pub sort_property: SortProperty,

    /// Output format for the extension list
    ///
    /// newline — one extension per line (default, human-readable)
    /// comma   — comma-separated list (useful for scripting)
    #[clap(long, value_enum, global = true, default_value_t = ListPresentation::Newline)]
    pub presentation: ListPresentation,
}

#[derive(Debug, Args)]
pub struct GlobalOptions {
    /// IF system to target (default: auto-detect from current directory)
    ///
    /// pif inspects the current directory for recognised file extensions and
    /// picks the most likely system:
    ///
    ///   Makefile / .t3m   → tads3
    ///   .inf              → inform6
    ///   .i7x / .materials → inform
    ///   .dg               → dialog
    ///   .hug              → hugo
    ///   .zil              → zil
    ///
    /// Use --system=all to list or install across every known system at once.
    #[clap(long, short = 's', value_enum, global = true, default_value_t = InteractiveFictionSystem::Auto)]
    pub system: InteractiveFictionSystem,

    /// Colour output control
    ///
    ///   auto   — use colour when stdout is a terminal (default)
    ///   always — force colour even when piped
    ///   never  — disable colour entirely
    #[clap(long, value_enum, global = true, default_value_t = Color::Auto)]
    pub color: Color,

    /// Verbosity level (1–3)
    ///
    ///   1 — errors and warnings only
    ///   2 — normal output (default)
    ///   3 — detailed / debug output
    ///
    /// The default can be changed permanently with `pif config verbose set`.
    #[clap(short, long, global = true, default_value_t = *crate::config::VERBOSE_DEFAULT)]
    pub verbose: usize,
}


#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum InteractiveFictionSystem {
    Auto,
    All,
    Tads3,
    Tads2,
    Dialog,
    Inform,
    Inform6,
    Hugo,
    Zil,
    Unknown,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum Color {
    Always,
    Auto,
    Never,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum InstallationDirectory {
    Libs,
}


#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum ListPresentation {
    Newline,
    Comma,
}


#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum OrderingDirection {
    Ascending,
    Descending
}


#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum SortProperty {
    Name,
    Author,
    Date
}

#[derive(Debug, Args)]
pub struct TagsCommand {}

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[clap(subcommand)]
    pub action: ConfigAction,
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Manage per-system installation directories
    ///
    /// Sets or resets the directory where pif installs extensions for a given
    /// IF system. The configured path is stored in the pif config file and
    /// used as the default target for every subsequent `pif install`.
    Dir(DirCommand),

    /// Manage the verbosity level
    ///
    /// Persists the verbosity preference to the config file so that the chosen
    /// level becomes the default for all future pif invocations. Override for
    /// a single run with the global --verbose flag.
    Verbose(VerboseCommand),

    /// Manage the systems filter (restricts Auto mode to specific systems)
    ///
    /// When --system=auto (the default), pif normally considers all known
    /// systems. Adding a systems filter tells Auto mode to behave as if only
    /// the listed systems exist — useful when you work with multiple IF systems
    /// but want installs scoped to a subset.
    Systems(SystemsCommand),

    /// Manage per-system version constraints
    ///
    /// Version specs are prefix strings (e.g. "i10", "i11.0") that restrict
    /// which extension versions are considered compatible for a given system.
    /// Matching is done as a string prefix against each version string in the
    /// registry.
    ///
    /// Example: adding "i10" for inform means only versions starting with
    /// "i10" (like "i10.1", "i10.2") will be offered when installing.
    Versions(VersionsCommand),

    /// Display the current config file with syntax highlighting
    Show,
}

// ── dir ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct DirCommand {
    #[clap(subcommand)]
    pub action: DirAction,
}

#[derive(Debug, Subcommand)]
pub enum DirAction {
    /// Set the install directory for a system
    ///
    /// Writes the given path to the pif config file as the default install
    /// location for SYSTEM. Paths starting with ~/ are expanded to your home
    /// directory.
    ///
    /// Example: pif config dir set tads3 ~/MyGame/extensions
    Set(SetDirCommand),

    /// Reset the install directory to the built-in default
    ///
    /// Removes the configured path for SYSTEM from the config file so that
    /// pif falls back to prompting on the next install. Omit SYSTEM to remove
    /// all custom directory entries at once.
    Reset(ResetDirCommand),
}

// ── verbose ───────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct VerboseCommand {
    #[clap(subcommand)]
    pub action: VerboseAction,
}

#[derive(Debug, Subcommand)]
pub enum VerboseAction {
    /// Persist a verbosity level to the config file
    ///
    /// Levels:
    ///
    ///   1 — errors and warnings only
    ///   2 — normal output (default)
    ///   3 — detailed / debug output
    Set(SetVerboseCommand),

    /// Reset the verbosity level to the built-in default (2)
    Reset,
}

// ── systems ───────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct SystemsCommand {
    #[clap(subcommand)]
    pub action: SystemsAction,
}

#[derive(Debug, Subcommand)]
pub enum SystemsAction {
    /// Replace the entire systems filter with a new list
    ///
    /// Auto mode will only consider the systems listed here. Any previously
    /// configured systems are discarded.
    Set(SetSystemsCommand),

    /// Add one or more systems to the existing filter
    ///
    /// Appends to the current filter without removing existing entries.
    Add(SetSystemsCommand),

    /// Remove one or more systems from the filter
    ///
    /// The remaining entries in the filter are unchanged.
    Remove(SetSystemsCommand),

    /// Remove the entire systems filter
    ///
    /// After reset, Auto mode considers all known IF systems again.
    Reset,
}

// ── versions ──────────────────────────────────────────────────────────────────

#[derive(Debug, Args)]
pub struct VersionsCommand {
    #[clap(subcommand)]
    pub action: VersionsAction,
}

#[derive(Debug, Subcommand)]
pub enum VersionsAction {
    /// Replace version specs for a system with a new list
    ///
    /// Discards any previously configured version specs for SYSTEM and stores
    /// the given list instead.
    Set(SetVersionsCommand),

    /// Add one or more version specs for a system
    ///
    /// Appends to the current version spec list for SYSTEM without removing
    /// existing entries.
    Add(SetVersionsCommand),

    /// Remove one or more version specs for a system
    ///
    /// The remaining version specs for SYSTEM are unchanged.
    Remove(SetVersionsCommand),

    /// Remove all version specs for a system, or all systems if omitted
    ///
    /// After reset, no version filtering is applied when installing extensions
    /// for the affected system(s).
    Reset(ResetVersionsCommand),
}

#[derive(Debug, Args)]
pub struct ResetDirCommand {
    /// IF system name (e.g. tads3, dialog, inform, inform6). Omit to reset all.
    #[arg(value_name = "SYSTEM")]
    pub sys: Option<String>,
}

#[derive(Debug, Args)]
pub struct SetVerboseCommand {
    /// Verbosity level to persist (1–3)
    pub level: usize,
}

#[derive(Debug, Args)]
pub struct SetDirCommand {
    /// IF system name (e.g. tads3, dialog, inform, inform6)
    #[arg(value_name = "SYSTEM")]
    pub sys: String,

    /// Directory path where extensions for this system will be installed
    ///
    /// Supports ~/ prefix for home directory expansion.
    pub directory: String,
}

#[derive(Debug, Args)]
pub struct RegistryCommand {
    #[clap(subcommand)]
    pub action: RegistryAction,
}

#[derive(Debug, Subcommand)]
pub enum RegistryAction {
    /// Print all recorded extension installations
    ///
    /// Shows the name, installed version, and on-disk path for every extension
    /// that pif has installed on this machine.
    List,

    /// Remove a registry entry by extension name
    ///
    /// Deletes the record from the pif registry without touching the files on
    /// disk. Use this to tell pif to forget an installation (e.g. before
    /// reinstalling to a different path).
    Remove(RegistryRemoveCommand),

    /// Remove registry entries whose install paths no longer exist on disk
    ///
    /// Scans every recorded installation path and removes entries whose
    /// directory or file is gone. Useful after manually deleting extensions.
    Clean,
}

#[derive(Debug, Args)]
pub struct SetSystemsCommand {
    /// System names to include (e.g. tads3 inform6 inform)
    #[arg(value_name = "SYSTEM", required = true)]
    pub systems: Vec<String>,
}

#[derive(Debug, Args)]
pub struct SetVersionsCommand {
    /// IF system name (e.g. inform, tads3)
    #[arg(value_name = "SYSTEM")]
    pub sys: String,

    /// Version prefix strings to match (e.g. i10 i11.0)
    ///
    /// Each string is matched as a prefix against version entries in the
    /// registry. For example, "i10" matches "i10.1" and "i10.2.3" but not
    /// "i11.0".
    #[arg(value_name = "VERSION", required = true)]
    pub versions: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ResetVersionsCommand {
    /// IF system name. Omit to reset version specs for all systems.
    #[arg(value_name = "SYSTEM")]
    pub sys: Option<String>,
}

#[derive(Debug, Args)]
pub struct RegistryRemoveCommand {
    /// Name of the extension to remove from the registry
    pub name: String,

    /// Restrict removal to a specific install path (optional)
    ///
    /// When an extension has been installed to multiple paths, this narrows
    /// the removal to the matching entry only.
    #[arg(short, long)]
    pub path: Option<String>,
}
