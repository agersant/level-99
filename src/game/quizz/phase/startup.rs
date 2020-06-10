use std::time::Duration;

use crate::game::quizz::State;
use crate::output::{OutputPipe, Payload};

#[derive(Debug)]
pub struct StartupState {}

impl StartupState {
    pub fn new() -> Self {
        StartupState {}
    }
}

impl State for StartupState {
    fn on_tick(&mut self, _output_pipe: &mut OutputPipe, _dt: Duration) {}

    fn on_begin(&mut self, _output_pipe: &mut OutputPipe) {}

    fn on_end(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::Text("The quizz begins!".into()));
    }

    fn is_over(&self) -> bool {
        true
    }
}
