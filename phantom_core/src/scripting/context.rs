use crate::{input::Input, scripting::time::Time};

pub struct ScriptContext {
    pub input: Input,
    pub time: Time,
}

impl Default for ScriptContext {
    fn default() -> Self {
        Self {
            input: Input::default(),
            time: Time::default(),
        }
    }
}
