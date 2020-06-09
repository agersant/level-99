use std::time::Duration;

use crate::game::quizz::{State, Transition};
use crate::output::OutputPipe;

#[derive(Debug)]
pub struct StartupState {}

impl StartupState {
    pub fn new() -> Self {
        StartupState {}
    }
}

impl State for StartupState {
    fn tick(&mut self, _output_pipe: &mut OutputPipe, _dt: Duration) -> Option<Transition> {
        None
    }

    fn begin(&mut self, _output_pipe: &mut OutputPipe) {}

    fn end(&mut self, _output_pipe: &mut OutputPipe) {}
}
