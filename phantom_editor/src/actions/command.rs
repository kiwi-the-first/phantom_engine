use std::sync::{Arc, Mutex};

use crate::context::EditorContext;

pub trait Command: Send + Sync {
    fn execute(&mut self, ctx: &Arc<Mutex<EditorContext>>);
    fn undo(&mut self, ctx: &Arc<Mutex<EditorContext>>);
}
