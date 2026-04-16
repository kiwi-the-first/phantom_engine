use crate::args::{Cli, Commands};
use clap::Parser;
mod args;
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create(args) => {
            println!("{}", args.name);
        }
    }
}
