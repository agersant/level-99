use std::time::Duration;

use crate::game::quiz::State;
use crate::output::{GameOutput, Message, Recipient};
use crate::preload;
use crate::preload::{PreloadHandle, PreloadState};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct StartupState<O> {
    time_elapsed: Duration,
    time_to_wait: Duration,
    preload_handle: PreloadHandle,
    preload_state: PreloadState,
    output: O,
}

impl<O: GameOutput> StartupState<O> {
    pub fn new(duration: Duration, song_urls: &Vec<String>, output: O) -> Self {
        let preload_handle = preload::preload_songs(song_urls).unwrap(); // todo
        StartupState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
            preload_handle,
            preload_state: PreloadState::InProgress,
            output,
        }
    }

    pub fn preload_succeeded(&self) -> bool {
        match self.preload_state {
            PreloadState::Success => true,
            _ => false,
        }
    }
}

impl<O: GameOutput> State for StartupState<O> {
    fn on_tick(&mut self, dt: Duration) {
        self.time_elapsed += dt;
        self.preload_state = self.preload_handle.get_state();
    }

    fn on_begin(&mut self) {
        self.output.say(&Recipient::AllTeams, &Message::QuizRules);
    }

    fn on_end(&mut self) {}

    fn is_over(&self) -> bool {
        let waited = self.time_elapsed >= self.time_to_wait;
        match self.preload_state {
            PreloadState::InProgress => false,
            PreloadState::Failure => true,
            PreloadState::Success => waited,
        }
    }
}
