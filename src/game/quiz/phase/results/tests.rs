use super::*;
use parking_lot::RwLock;
use std::sync::Arc;
use crate::game::team::Team;
use crate::game::{TeamId, TeamsHandle};
use crate::output::mock::MockGameOutput;

// #[test]
// fn prints_rules() {
//     let duration = Duration::from_secs(10);
//     let mut output = MockGameOutput::new();
//     let mut state = StartupState::new(duration, &Vec::new(), output.clone());
//     assert!(output.flush().is_empty());
//     state.on_begin();
//     assert_eq!(output.flush(), [Message::QuizRules]);
// }

#[test]
fn plays_sfx_congrats() {
    let mut output = MockGameOutput::new();

    let teams: TeamsHandle =  Arc::new(RwLock::new(
        ["red", "green", "blue"]
            .iter()
            .map(|team_name| {
                let team_string = team_name.to_string();
                let team_id = TeamId::TeamName(team_string);
                Team::new(team_id)
            })
            .collect()
    ));

    let mut state = ResultsState::new(teams, output.clone());
    assert!(output.flush().is_empty());
    state.on_begin();
    assert_eq!(output.flush(), [Message::GameResults(TeamId::TeamName("blue".to_string()))]);
}