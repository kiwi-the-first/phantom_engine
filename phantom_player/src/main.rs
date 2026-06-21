#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use anyhow::Result;
use phantom_runtime::{self, App};

fn main() -> Result<()> {
    App::run()?;
    Ok(())
}
