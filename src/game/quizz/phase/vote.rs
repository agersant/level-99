use std::time::Duration;

use crate::game::quizz::{State, Transition};
use crate::output::OutputPipe;

#[derive(Debug)]
pub struct VoteState {
    time_elapsed: Duration,
    time_to_wait: Duration,
}

impl VoteState {
    pub fn new(duration: Duration) -> Self {
        VoteState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
        }
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}

impl State for VoteState {
    fn tick(&mut self, _output_pipe: &mut OutputPipe, dt: Duration) -> Option<Transition> {
        self.time_elapsed += dt;
        if !self.is_over() {
            None
        } else {
            Some(Transition::ToQuestionPhase)
        }
    }

    fn begin(&mut self, _output_pipe: &mut OutputPipe) {}

    fn end(&mut self, _output_pipe: &mut OutputPipe) {}
}
