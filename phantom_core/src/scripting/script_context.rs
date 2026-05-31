use crate::{
    input::InputContext,
    time::{time_context::TimeContext, time_system::TimeSystem},
};

pub struct ScriptContext<'c> {
    pub input: &'c InputContext,
    pub time: &'c TimeContext,
}
