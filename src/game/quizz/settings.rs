use std::time::Duration;

#[derive(Debug)]
pub struct Settings {
    pub cooldown_duration: Duration,
    pub vote_duration: Duration,
    pub question_duration: Duration,
    pub max_vote_options: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            cooldown_duration: Duration::from_secs(15),
            vote_duration: Duration::from_secs(30),
            question_duration: Duration::from_secs(90),
            max_vote_options: 5,
        }
    }
}
