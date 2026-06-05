use egui::{Context, Key, Modifiers};

use crate::{actions::Actions, context::EditorContext};

/// Editor-wide keyboard shortcuts (undo/redo, etc.). Consumes the matching key
/// events from egui so they don't also reach widgets, then applies them.
pub struct EditorShortcuts;

impl EditorShortcuts {
    pub fn handle(ctx: &Context, actions: &mut Actions, editor: &mut EditorContext) {
        let (undo, redo) = ctx.input_mut(|i| {
            let redo = i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
            let undo = i.consume_key(Modifiers::COMMAND, Key::Z);
            (undo, redo)
        });

        if undo {
            actions.undo(editor);
        }
        if redo {
            actions.redo(editor);
        }
    }
}
