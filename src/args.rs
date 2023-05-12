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
    /// Get an update of all extensions
    Update(UpdateCommand),

    /// Install an extension
    Install(InstallCommand),

    /// Lists available extensions
    List(ListCommand),
}

#[derive(Debug, Args)]
pub struct UpdateCommand {
}


#[derive(Debug, Args)]
pub struct InstallCommand {
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
    /// Color
    #[clap(long, value_enum, global = true, default_value_t = Color::Auto)]
    pub color: Color,

    /// Verbosity level 1-3 (TODO)
    #[clap(short, long, global = true, default_value = "2")]
    pub verbose: Option<usize>,
}

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum Color {
    Always,
    Auto,
    Never,
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