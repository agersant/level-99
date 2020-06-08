use std::time::Duration;

pub struct Settings {
    pub cooldown_duration: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            cooldown_duration: Duration::from_secs(10),
        }
    }
}
