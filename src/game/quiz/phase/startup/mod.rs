use std::collections::HashSet;
use std::time::Duration;

use crate::game::quiz::definition::Question;
use crate::game::quiz::State;
use crate::output::{GameOutput, Message, Recipient};
use crate::preload;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct StartupState<O> {
    time_elapsed: Duration,
    time_to_wait: Duration,
    song_urls: Vec<String>,
    output: O,
}

impl<O: GameOutput> StartupState<O> {
    pub fn new(duration: Duration, questions: &HashSet<Question>, output: O) -> Self {
        StartupState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
            song_urls: questions.iter().map(|q| q.url.clone()).collect(),
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
        preload::preload_songs(&self.song_urls).ok();
    }

    fn on_end(&mut self) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
