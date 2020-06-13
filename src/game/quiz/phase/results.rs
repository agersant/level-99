use std::path::Path;
use std::time::Duration;

use crate::game::quiz::State;
use crate::game::TeamsHandle;
use crate::output::{OutputPipe, Recipient};

const SFX_CONGRATS: &'static str = "assets/congrats.wav";

#[derive(Debug)]
pub struct ResultsState {
    teams: TeamsHandle,
}

impl ResultsState {
    pub fn new(teams: TeamsHandle) -> Self {
        ResultsState { teams }
    }
}

impl State for ResultsState {
    fn on_tick(&mut self, _output_pipe: &mut OutputPipe, _dt: Duration) {}

    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        if let Some(winning_team) = self.teams.read().iter().max_by_key(|t| t.score) {
            output_pipe.play_file_audio(Path::new(SFX_CONGRATS)).ok();
            let message = format!(
                "🎊🎊 **TEAM {} WINS IT ALL!** 🎊🎊",
                winning_team.get_display_name()
            )
            .to_uppercase();
            output_pipe.say(&Recipient::AllTeams, &message);
        }
    }

    fn on_end(&mut self, _output_pipe: &mut OutputPipe) {}

    fn is_over(&self) -> bool {
        false
    }
}
