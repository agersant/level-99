use std::time::Duration;

pub struct Settings {
    pub cooldown_duration: Duration,
    pub vote_duration: Duration,
    pub question_duration: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            cooldown_duration: Duration::from_secs(10),
            vote_duration: Duration::from_secs(10),
            question_duration: Duration::from_secs(90),
        }
    }
}
