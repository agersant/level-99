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
    let output = MockGameOutput::new(teams.clone());

    let mut state = ResultsState::new(teams, output.clone());
    state.on_begin();

    assert!(output.is_playing_audio(Path::new(SFX_CONGRATS)));
}

#[test]
fn announces_winning_team() {
    let team_id = TeamId::TeamName("blue".to_owned());
    let teams = vec![Team::new(team_id.clone())];
    let teams: TeamsHandle = Arc::new(RwLock::new(teams));
    let output = MockGameOutput::new(teams.clone());

    let mut state = ResultsState::new(teams, output.clone());
    state.on_begin();

    let message = Message::GameResults(team_id.clone());
    assert!(output.contains_message(&team_id, &message));
}
