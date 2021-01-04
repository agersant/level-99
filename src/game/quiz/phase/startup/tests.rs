use super::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::game::{Team, TeamId, TeamsHandle};
use crate::output::mock::MockGameOutput;

#[test]
fn ends_after_duration() {
    let duration = Duration::from_secs(10);
    let teams: TeamsHandle = Arc::new(RwLock::new(vec![]));
    let output = MockGameOutput::new(teams.clone());
    let mut state = StartupState::new(duration, &Vec::new(), output.clone());
    assert!(!state.is_over());
    state.on_begin();
    assert!(!state.is_over());
    state.on_tick(Duration::from_secs(5));
    assert!(!state.is_over());

    let start_time = Instant::now();
    let tick_duration = Duration::from_millis(100);
    loop {
        state.on_tick(tick_duration);
        if state.is_over() {
            break;
        }
        if Instant::now().duration_since(start_time) > Duration::from_secs(5) {
            panic!("Timed out waiting for startup phase to end");
        }
        std::thread::sleep(tick_duration);
    }
}

#[test]
fn prints_rules() {
    let team_id = TeamId::TeamName("blue".to_owned());
    let duration = Duration::from_secs(10);
    let teams = vec![Team::new(team_id.clone())];
    let teams: TeamsHandle = Arc::new(RwLock::new(teams));
    let output = MockGameOutput::new(teams.clone());
    let mut state = StartupState::new(duration, &Vec::new(), output.clone());
    state.on_begin();
    assert!(output.contains_message(&team_id, &Message::QuizRules));
}
