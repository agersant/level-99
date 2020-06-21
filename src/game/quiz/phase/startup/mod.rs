use std::time::Duration;

use crate::game::quiz::State;
use crate::output::{GameOutput, Message, Recipient};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct StartupState<O> {
    time_elapsed: Duration,
    time_to_wait: Duration,
    output: O,
}

impl<O: GameOutput> StartupState<O> {
    pub fn new(duration: Duration, output: O) -> Self {
        StartupState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
            output,
        }
    }
}

impl<O: GameOutput> State for StartupState<O> {
    fn on_tick(&mut self, dt: Duration) {
        self.time_elapsed += dt;
    }

    fn on_begin(&mut self) {
        self.output.say(&Recipient::AllTeams, &Message::QuizRules);
    }

    fn on_end(&mut self) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
