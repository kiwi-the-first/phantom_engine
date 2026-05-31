use std::time::Instant;

use crate::time::time_context::TimeContext;

pub struct TimeSystem {
    pub time_ctx: TimeContext,
    last_frame: Instant,
}

impl Default for TimeSystem {
    fn default() -> Self {
        Self {
            time_ctx: TimeContext::default(),
            last_frame: std::time::Instant::now(),
        }
    }
}

impl TimeSystem {
    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        self.time_ctx.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.time_ctx.elapsed += self.time_ctx.delta;
        self.last_frame = now;
    }
}
