use anyhow::*;
use std::path::Path;
use std::time::Duration;

use crate::game::definition::*;
use crate::game::settings::*;

enum QuizzStep {
    Cooldown(CooldownState),
    Vote,
    Question,
}

struct CooldownState {
    time_elapsed: Duration,
    time_to_wait: Duration,
}

impl CooldownState {
    pub fn new(duration: Duration) -> Self {
        CooldownState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
        }
    }
}

pub struct Quizz {
    settings: Settings,
    remaining_questions: Vec<Question>,
    current_step: QuizzStep,
}

impl Quizz {
    fn new(definition: QuizzDefinition) -> Quizz {
        let settings: Settings = Default::default();
        Quizz {
            remaining_questions: definition.get_questions().clone(),
            current_step: QuizzStep::Cooldown(CooldownState::new(settings.cooldown_duration)),
            settings,
        }
    }

    pub fn load(source: &Path) -> Result<Quizz> {
        let definition = QuizzDefinition::open(source)?;
        Ok(Quizz::new(definition))
    }
}
