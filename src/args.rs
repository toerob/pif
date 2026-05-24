use clap::{Args, Parser, Subcommand, ValueEnum};


#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct InteractiveFictionToolArgs {
    #[clap(subcommand)]
    pub menu: MenuSubCommand,

    #[clap(flatten)]
    pub global_options: GlobalOptions,
}

#[derive(Debug, Subcommand)]
pub enum MenuSubCommand {
    /// Show information about an extension
    Info(InfoCommand),

    /// Get an update of all extensions
    Update(UpdateCommand),

    /// Install an extension
    Install(InstallCommand),

    /// Lists available extensions
    List(ListCommand),

    /// Publish an extension to the pif index
    Publish(PublishCommand),

    /// List all available tags
    Tags(TagsCommand),

    /// Search extensions by name and description
    Search(SearchCommand),

    /// Manage pif configuration
    Config(ConfigCommand),

    /// Manage the local extension registry
    Registry(RegistryCommand),
}

#[derive(Debug, Args)]
pub struct UpdateCommand {
}

#[derive(Debug, Args)]
pub struct PublishCommand {
    /// Path to the extension directory
    #[arg(default_value = ".")]
    pub directory: String,
}


#[derive(Debug, Args)]
pub struct InstallCommand {
    /// Names of the extensions together with a specific version, colon-separated. 
    /// e.g t3cartographer:1.0 conspace:2.1
    /// if version is left out LATEST will be used as default
    pub names: Vec<String>,

    #[clap(flatten)]
    pub install_options: InstallOptions,
}

#[derive(Debug, Args)]
pub struct InfoCommand {
    /// package names to retrieve info about
    pub name: Vec<String>,

    /// Filter by author (fuzzy search)
    #[clap(long, short, global = false)]
    pub author: Option<String>,

    /// Filter by keyword (fuzzy search)
    #[clap(long, short, global = false)]
    pub keyword: Option<String>,

    /// Filter by tag (exact match)
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
    /// Search query matched against name and description
    pub query: String,

    #[clap(flatten)]
    pub list_options: ListOptions,
}

#[derive(Debug, Args)]
pub struct InstallOptions {
    /// Directory where the extensions gets installed
    #[arg(short = 'd', long = "directory", value_name = "FOLDER")]
    pub installation_directory: Option<String>,
}


#[derive(Debug, Args)]
pub struct ListOptions {
    /// Filter by author (fuzzy search)
    #[clap(long, short, global = false)]
    pub author: Option<String>,

    /// Filter by keyword (fuzzy search)
    #[clap(long, short, global = false)]
    pub keyword: Option<String>,

    /// Filter by tag (exact match)
    #[clap(long, short = 't', global = false)]
    pub tag: Option<String>,

    /// List order direction
    #[clap(long, value_enum, global = true, default_value_t = OrderingDirection::Descending)]
    pub ordering_direction: OrderingDirection,

    /// Sort 
    #[clap(long, value_enum, global = true, default_value_t = SortProperty::Name)]
    pub sort_property: SortProperty,


    /// Presentation
    #[clap(long, value_enum, global = true, default_value_t = ListPresentation::Newline)]
    pub presentation: ListPresentation,
}

#[derive(Debug, Args)]
pub struct GlobalOptions {
    /// System
    #[clap(long, short = 's', value_enum, global = true, default_value_t = InteractiveFictionSystem::Auto)]
    pub system: InteractiveFictionSystem,

    /// Color
    #[clap(long, value_enum, global = true, default_value_t = Color::Auto)]
    pub color: Color,

    /// Verbosity level 1-3
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
    Dir(DirCommand),

    /// Manage the verbosity level
    Verbose(VerboseCommand),

    /// Manage the systems filter (restricts Auto mode to specific systems)
    Systems(SystemsCommand),

    /// Manage per-system version specs
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
    Set(SetDirCommand),

    /// Reset the install directory to default (omit system to reset all)
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
    /// Set the verbosity level (1-3)
    Set(SetVerboseCommand),

    /// Reset to the built-in default (2)
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
    /// Replace the systems filter with a new list
    Set(SetSystemsCommand),

    /// Add one or more systems to the filter
    Add(SetSystemsCommand),

    /// Remove one or more systems from the filter
    Remove(SetSystemsCommand),

    /// Remove the entire systems filter
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
    Set(SetVersionsCommand),

    /// Add one or more version specs for a system
    Add(SetVersionsCommand),

    /// Remove one or more version specs for a system
    Remove(SetVersionsCommand),

    /// Remove all version specs for a system, or all systems if omitted
    Reset(ResetVersionsCommand),
}

#[derive(Debug, Args)]
pub struct ResetDirCommand {
    /// System name (tads3, dialog, inform, inform6). Omit to reset all.
    #[arg(value_name = "SYSTEM")]
    pub sys: Option<String>,
}

#[derive(Debug, Args)]
pub struct SetVerboseCommand {
    /// Verbosity level (1-3)
    pub level: usize,
}

#[derive(Debug, Args)]
pub struct SetDirCommand {
    /// System name (tads3, dialog, inform, inform6)
    #[arg(value_name = "SYSTEM")]
    pub sys: String,

    /// Directory path (supports ~/)
    pub directory: String,
}

#[derive(Debug, Args)]
pub struct RegistryCommand {
    #[clap(subcommand)]
    pub action: RegistryAction,
}

#[derive(Debug, Subcommand)]
pub enum RegistryAction {
    /// List all recorded extension installations
    List,

    /// Remove a registry entry by extension name
    Remove(RegistryRemoveCommand),

    /// Remove registry entries whose install paths no longer exist
    Clean,
}

#[derive(Debug, Args)]
pub struct SetSystemsCommand {
    /// System names to restrict to (e.g. tads3 inform6 inform)
    #[arg(value_name = "SYSTEM", required = true)]
    pub systems: Vec<String>,
}

#[derive(Debug, Args)]
pub struct SetVersionsCommand {
    /// System name (e.g. inform, tads3)
    #[arg(value_name = "SYSTEM")]
    pub sys: String,

    /// Version matcher strings, prefix-matched (e.g. i10 i11.0)
    #[arg(value_name = "VERSION", required = true)]
    pub versions: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ResetVersionsCommand {
    /// System name. Omit to reset all systems.
    #[arg(value_name = "SYSTEM")]
    pub sys: Option<String>,
}

#[derive(Debug, Args)]
pub struct RegistryRemoveCommand {
    /// Name of the extension to remove from the registry
    pub name: String,

    /// Restrict removal to this specific install path (optional)
    #[arg(short, long)]
    pub path: Option<String>,
}