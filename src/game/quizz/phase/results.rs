use std::time::Duration;

use crate::game::quizz::{State, Transition};
use crate::output::OutputPipe;

#[derive(Debug)]
pub struct ResultsState {}

impl ResultsState {
    pub fn new() -> Self {
        ResultsState {}
    }
}

impl State for ResultsState {
    fn tick(&mut self, _output_pipe: &mut OutputPipe, _dt: Duration) -> Option<Transition> {
        None
    }

    fn begin(&mut self, _output_pipe: &mut OutputPipe) {}

    fn end(&mut self, _output_pipe: &mut OutputPipe) {}
}
