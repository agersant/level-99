use std::time::Duration;

#[derive(Debug)]
pub struct Settings {
    pub startup_duration: Duration,
    pub vote_duration: Duration,
    pub wager_duration: Duration,
    pub question_duration: Duration,
    pub cooldown_duration: Duration,
    pub max_vote_options: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            startup_duration: Duration::from_secs(30),
            vote_duration: Duration::from_secs(15),
            wager_duration: Duration::from_secs(90),
            question_duration: Duration::from_secs(90),
            cooldown_duration: Duration::from_secs(5),
            max_vote_options: 6,
        }
    }
}
