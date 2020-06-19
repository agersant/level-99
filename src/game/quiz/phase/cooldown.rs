use std::time::Duration;

use crate::game::quiz::State;

#[derive(Debug)]
pub struct CooldownState {
    time_elapsed: Duration,
    time_to_wait: Duration,
}

impl CooldownState {
    pub fn new(duration: Duration) -> Self {
        CooldownState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
        }
    }
}

impl State for CooldownState {
    fn on_begin(&mut self) {}

    fn on_tick(&mut self, dt: Duration) {
        self.time_elapsed += dt;
    }

    fn on_end(&mut self) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
