use crate::actions::command::Command;
use log::*;

pub struct Actions {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl Actions {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn do_command(&mut self, mut command: Box<dyn Command>) {
        command.execute();
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo();
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self) {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute();
            debug!("REDOING!");
            self.undo_stack.push(command);
        }
    }
}
