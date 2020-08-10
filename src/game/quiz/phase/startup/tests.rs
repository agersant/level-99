use super::*;
use std::time::{Duration, Instant};

use crate::output::mock::MockGameOutput;

#[test]
fn ends_after_duration() {
    let duration = Duration::from_secs(10);
    let output = MockGameOutput::new();
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
    let duration = Duration::from_secs(10);
    let mut output = MockGameOutput::new();
    let mut state = StartupState::new(duration, &Vec::new(), output.clone());
    assert!(output.flush().is_empty());
    state.on_begin();
    assert_eq!(output.flush(), [Message::QuizRules]);
}
