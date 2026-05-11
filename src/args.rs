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
    #[clap(long, value_enum, global = true, default_value_t = InteractiveFictionSystem::Auto)]
    pub system: InteractiveFictionSystem,

    /// Color
    #[clap(long, value_enum, global = true, default_value_t = Color::Auto)]
    pub color: Color,

    /// Verbosity level 1-3
    #[clap(short, long, global = true, default_value = "2")]
    pub verbose: Option<usize>,
}


#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum InteractiveFictionSystem {
    Auto,
    Tads3,
    Dialog,
    Inform6,
    Inform7,
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
pub struct RegistryRemoveCommand {
    /// Name of the extension to remove from the registry
    pub name: String,

    /// Restrict removal to this specific install path (optional)
    #[arg(short, long)]
    pub path: Option<String>,
}