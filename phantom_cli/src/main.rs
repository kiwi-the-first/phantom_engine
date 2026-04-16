use std::{
    env::current_dir,
    path::{Path, PathBuf},
    process::Command,
};

use crate::args::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use phantom_project::create::{self, create_project};
mod args;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create(args) => {
            let path = PathBuf::from(&args.name);
            let name = path.file_name().unwrap().to_str().unwrap();
            create_project(name.to_string(), path)?;
        }
        Commands::Edit(args) => {
            let path = PathBuf::from(current_dir().unwrap().join(args.path));
            Command::new("phantom_editor").arg(&path).spawn()?;
        }
    }

    Ok(())
}
