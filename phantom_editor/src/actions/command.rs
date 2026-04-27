pub trait Command: Send + Sync {
    fn execute(&mut self);
    fn undo(&mut self);
}
