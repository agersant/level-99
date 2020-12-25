use super::*;
use crate::game::team::Team;
use crate::game::{TeamId, TeamsHandle};
use crate::output::mock::MockGameOutput;
use parking_lot::RwLock;
use std::sync::Arc;

#[test]
fn plays_sfx_congrats() {
    let teams = vec![Team::new(TeamId::TeamName("blue".to_owned()))];
    let teams: TeamsHandle = Arc::new(RwLock::new(teams));
    let mut output = MockGameOutput::new(teams.clone());

    let mut state = ResultsState::new(teams, output.clone());
    assert!(output.flush().is_empty());
    state.on_begin();

    assert!(output.contains_audio(Path::new(SFX_CONGRATS)));
}

#[test]
fn announces_winning_team() {
    let teams = vec![Team::new(TeamId::TeamName("blue".to_owned()))];
    let teams: TeamsHandle = Arc::new(RwLock::new(teams));
    let mut output = MockGameOutput::new(teams.clone());

    let mut state = ResultsState::new(teams, output.clone());
    assert!(output.flush().is_empty());
    state.on_begin();

    let message = Message::GameResults(TeamId::TeamName("blue".to_string()));
    assert!(output.contains_message(&message));
}
