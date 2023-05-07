use clap::{
  Args,
  ArgMatches,
  Parser,
  Subcommand,

   ValueEnum
};

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
  /// Install an extension
  Install(InstallCommand),
  
  /// Lists available extensions
  List(ListCommand)
}

#[derive(Debug, Args)]
pub struct InstallCommand {
  pub name: String
}





#[derive(Debug, Subcommand)]
pub enum ListSubCommand {
  List(ListCommand)
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
}



#[derive(Debug, Args)]
pub struct GlobalOptions {
    /// Color
     #[clap(long, value_enum, global = true, default_value_t = Color::Auto)]
     color: Color,

    /// Verbosity level 1-3 (TODO)
     #[clap(short, long, global = true, default_value = "1" )]
     verbose: Option<usize>,     
}

#[derive(Clone, Debug, ValueEnum)]
enum Color {
    Always,
    Auto,
    Never,
}