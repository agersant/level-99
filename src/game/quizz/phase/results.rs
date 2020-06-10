use std::time::Duration;

use crate::game::quizz::State;
use crate::output::OutputPipe;

#[derive(Debug)]
pub struct ResultsState {}

impl ResultsState {
    pub fn new() -> Self {
        ResultsState {}
    }
}

impl State for ResultsState {
    fn on_tick(&mut self, _output_pipe: &mut OutputPipe, _dt: Duration) {}

    fn on_begin(&mut self, _output_pipe: &mut OutputPipe) {}

    fn on_end(&mut self, _output_pipe: &mut OutputPipe) {}

    fn is_over(&self) -> bool {
        false
    }
}
