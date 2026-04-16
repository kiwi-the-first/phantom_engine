use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Create a new project
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project
    Create(CreateArgs),
    Edit(EditArgs),
}

#[derive(Args)]
pub struct CreateArgs {
    /// Name of project
    pub name: String,
}

#[derive(Args)]
pub struct EditArgs {
    /// Name of project
    pub path: String,
}
