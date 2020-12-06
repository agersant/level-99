use std::path::Path;
use std::time::Duration;

use crate::game::quiz::State;
use crate::game::TeamsHandle;
use crate::output::{GameOutput, Message, Recipient};

const SFX_CONGRATS: &'static str = "assets/congrats.wav";

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct ResultsState<O> {
    teams: TeamsHandle,
    output: O,
}

impl<O> ResultsState<O> {
    pub fn new(teams: TeamsHandle, output: O) -> Self {
        ResultsState { teams, output }
    }
}

impl<O: GameOutput> State for ResultsState<O> {
    fn on_tick(&mut self, _dt: Duration) {}

    fn on_begin(&mut self) {
        if let Some(winning_team) = self.teams.read().iter().max_by_key(|t| t.score) {
            self.output.play_file_audio(Path::new(SFX_CONGRATS)).ok();
            self.output.say(
                &Recipient::AllTeams,
                &Message::GameResults(winning_team.id.clone()),
            );
        }
    }

    fn on_end(&mut self) {}

    fn is_over(&self) -> bool {
        false
    }
}
