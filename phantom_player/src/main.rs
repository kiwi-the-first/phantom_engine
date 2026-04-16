use anyhow::Result;
use phantom_runtime::{self, App};
fn main() -> Result<()> {
    App::run()?;
    Ok(())
}
