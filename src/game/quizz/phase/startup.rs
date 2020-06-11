use std::time::Duration;

use crate::game::quizz::State;
use crate::output::{OutputPipe, Payload};

#[derive(Clone, Debug)]
pub struct StartupState {
    time_elapsed: Duration,
    time_to_wait: Duration,
}

impl StartupState {
    pub fn new(duration: Duration) -> Self {
        StartupState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
        }
    }
}

impl State for StartupState {
    fn on_tick(&mut self, _output_pipe: &mut OutputPipe, dt: Duration) {
        self.time_elapsed += dt;
    }

    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::Text("The quizz is about to begin!\n\n**📋 Rules**\n- For each song, you can submit your team's guess with the `!guess something` command.\n- Use your team channel to discuss and submit guesses away from prying eyes.\n- Your team gets less points if another team guessed first, and guessing wrong will deduct points!"
                .into(),
        ));
    }

    fn on_end(&mut self, _output_pipe: &mut OutputPipe) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
