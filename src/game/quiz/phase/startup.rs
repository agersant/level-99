use std::collections::HashSet;
use std::time::Duration;

use crate::game::quiz::definition::Question;
use crate::game::quiz::State;
use crate::output::{OutputPipe, Recipient};
use crate::preload;

#[derive(Clone, Debug)]
pub struct StartupState {
    time_elapsed: Duration,
    time_to_wait: Duration,
    song_urls: Vec<String>,
}

impl StartupState {
    pub fn new(duration: Duration, questions: &HashSet<Question>) -> Self {
        StartupState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
            song_urls: questions.iter().map(|q| q.url.clone()).collect(),
        }
    }
}

impl State for StartupState {
    fn on_tick(&mut self, _output_pipe: &mut OutputPipe, dt: Duration) {
        self.time_elapsed += dt;
    }

    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.say(&Recipient::AllTeams,"The quiz is about to begin!\n\n**📋 Rules**\n- For each song, your team can submit **one** guess using the `!guess something` command.\n- Guessing wrong will deduct the same amount of points you could have earned!\n- If you are not the first team to guess, point earned or deducted are halved.\n\n**🔥 Tips**\n- Answers are game names, not song titles or composers.\n- You can adjust the music volume by right clicking on the bot in the voice channel UI.\n- Sometimes it is wiser to not answer than to lose points!");
        preload::preload_songs(&self.song_urls).ok();
    }

    fn on_end(&mut self, _output_pipe: &mut OutputPipe) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
