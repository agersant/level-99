use std::collections::HashSet;
use std::time::Duration;

use crate::game::quiz::definition::Question;
use crate::game::quiz::State;
use crate::output::{OutputHandle, Recipient};
use crate::preload;

#[derive(Clone, Debug)]
pub struct StartupState {
    time_elapsed: Duration,
    time_to_wait: Duration,
    song_urls: Vec<String>,
    output: OutputHandle,
}

impl StartupState {
    pub fn new(duration: Duration, questions: &HashSet<Question>, output: OutputHandle) -> Self {
        StartupState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
            song_urls: questions.iter().map(|q| q.url.clone()).collect(),
            output,
        }
    }
}

impl State for StartupState {
    fn on_tick(&mut self, dt: Duration) {
        self.time_elapsed += dt;
    }

    fn on_begin(&mut self) {
        self.output.say(&Recipient::AllTeams,"The quiz is about to begin!\n\n**📋 Rules**\n- For each song, your team can submit **one** guess using the `!guess something` command.\n- Guessing wrong will deduct the same amount of points you could have earned!\n- If you are not the first team to guess, point earned or deducted are halved.\n\n**🔥 Tips**\n- Answers are game names, not song titles or composers.\n- You can adjust the music volume by right clicking on the bot in the voice channel UI.\n- Sometimes it is wiser to not answer than to lose points!");
        preload::preload_songs(&self.song_urls).ok();
    }

    fn on_end(&mut self) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
