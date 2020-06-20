use super::*;
use std::time::Duration;

#[test]
fn ends_after_duration() {
    let duration = Duration::from_secs(10);
    let mut state = CooldownState::new(duration);
    assert!(!state.is_over());
    state.on_begin();
    assert!(!state.is_over());
    state.on_tick(Duration::from_secs(5));
    assert!(!state.is_over());
    state.on_tick(Duration::from_secs(5));
    assert!(state.is_over());
}
