use crate::input::Input;

pub struct ScriptContext {
    pub input: Input,
}

impl Default for ScriptContext {
    fn default() -> Self {
        Self {
            input: Input::default(),
        }
    }
}
