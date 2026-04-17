use directories::ProjectDirs;

#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{env::current_dir, fs::OpenOptions, path::PathBuf, process::Command};

use crate::args::{Cli, Commands};
use anyhow::Result;
use clap::Parser;
use phantom_project::create::create_project;
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

            // Create log file for editor output
            let proj_dirs = ProjectDirs::from("", "phantom", "phantom")
                .expect("Failed to determine system directories");

            let log_path = proj_dirs.cache_dir().join("editor.log");
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let log_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&log_path)
                .ok();

            #[cfg(unix)]
            let child = {
                let mut cmd = Command::new("phantom_editor");
                cmd.arg(&path)
                    .stdin(std::process::Stdio::null())
                    .process_group(0); // Create new process group to detach from terminal

                if let Some(log) = log_file {
                    cmd.stdout(log.try_clone()?).stderr(log);
                } else {
                    cmd.stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                }

                cmd.spawn()?
            };

            #[cfg(not(unix))]
            let child = {
                let mut cmd = Command::new("phantom_editor");
                cmd.arg(&path).stdin(std::process::Stdio::null());

                if let Some(log) = log_file {
                    cmd.stdout(log.try_clone()?).stderr(log);
                } else {
                    cmd.stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                }

                cmd.spawn()?
            };

            println!("Editor launched! Logs: {}", log_path.display());

            // Explicitly don't wait for child
            drop(child); // Just drop the handle, don't wait

            std::process::exit(0);
        }
    }

    Ok(())
}
