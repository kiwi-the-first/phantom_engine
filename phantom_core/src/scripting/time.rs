use std::time::Instant;

pub struct Time {
    pub delta: f32,
    pub elapsed: f32, // total time since start
    last_frame: Instant,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
            last_frame: std::time::Instant::now(),
        }
    }
}

impl Time {
    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        self.delta = now.duration_since(self.last_frame).as_secs_f32();
        self.elapsed += self.delta;
        self.last_frame = now;
    }
}
