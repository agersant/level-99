use super::*;
use std::time::Duration;

use crate::output::mock::MockGameOutput;

#[test]
fn ends_after_duration() {
    let duration = Duration::from_secs(10);
    let questions = HashSet::new();
    let output = MockGameOutput::new();
    let mut state = StartupState::new(duration, &questions, output.clone());
    assert!(!state.is_over());
    state.on_begin();
    assert!(!state.is_over());
    state.on_tick(Duration::from_secs(5));
    assert!(!state.is_over());
    state.on_tick(Duration::from_secs(5));
    assert!(state.is_over());
}

#[test]
fn prints_rules() {
    let duration = Duration::from_secs(10);
    let questions = HashSet::new();
    let mut output = MockGameOutput::new();
    let mut state = StartupState::new(duration, &questions, output.clone());
    assert!(output.flush().is_empty());
    state.on_begin();
    assert_eq!(output.flush(), [Message::QuizRules]);
}
