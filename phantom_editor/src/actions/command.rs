use crate::context::EditorContext;

pub trait Command: Send + Sync {
    fn execute(&mut self, ctx: &mut EditorContext);
    fn undo(&mut self, ctx: &mut EditorContext);
}
