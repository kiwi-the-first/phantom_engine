use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PhantomProject {
    pub name: String,
    pub version: String,
    pub entry_world: PathBuf,
}
