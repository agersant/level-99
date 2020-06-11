use std::time::Duration;

use crate::game::quizz::State;
use crate::output::{OutputPipe, Recipient};

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
        output_pipe.say(&Recipient::AllTeams,"The quizz is about to begin!\n\n**📋 Rules**\n- For each song, your team can submit **one** guess using the `!guess something` command.\n- Guessing wrong will deduct the same amount of points you could have earned!\n- If you are not the first team to guess, point earned or deducted are halved.\n\n**🔥 Tips**\n- Answers are game names, not song titles or composers.\n- You can adjust the music volume by right clicking on the bot in the voice channel UI.\n- Sometimes it is wiser to not answer than to lose points!");
    }

    fn on_end(&mut self, _output_pipe: &mut OutputPipe) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
