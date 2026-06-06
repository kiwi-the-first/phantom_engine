pub struct TimeContext {
    /// Time elapsed since the last frame in seconds.
    pub delta: f32,
    /// Total time elapsed since the start of the game in seconds.
    pub elapsed: f32,
}

impl Default for TimeContext {
    fn default() -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
        }
    }
}
