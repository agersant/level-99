use std::path::Path;
use std::time::Duration;

use crate::game::quiz::State;
use crate::game::TeamsHandle;
use crate::output::{OutputHandle, Recipient};

const SFX_CONGRATS: &'static str = "assets/congrats.wav";

#[derive(Debug)]
pub struct ResultsState {
    teams: TeamsHandle,
    output: OutputHandle,
}

impl ResultsState {
    pub fn new(teams: TeamsHandle, output: OutputHandle) -> Self {
        ResultsState { teams, output }
    }
}

impl State for ResultsState {
    fn on_tick(&mut self, _dt: Duration) {}

    fn on_begin(&mut self) {
        if let Some(winning_team) = self.teams.read().iter().max_by_key(|t| t.score) {
            self.output.play_file_audio(Path::new(SFX_CONGRATS)).ok();
            let message = format!(
                "🎊🎊 **TEAM {} WINS IT ALL!** 🎊🎊",
                winning_team.get_display_name()
            )
            .to_uppercase();
            self.output.say(&Recipient::AllTeams, &message);
        }
    }

    fn on_end(&mut self) {}

    fn is_over(&self) -> bool {
        false
    }
}
