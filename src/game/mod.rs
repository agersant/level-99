use anyhow::*;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub mod pool;
mod quizz;

use crate::game::quizz::definition::QuizzDefinition;
use crate::game::quizz::Quizz;
use crate::output::OutputPipe;

enum Phase {
    Setup(SetupState),
    Quizz(QuizzState),
}

#[derive(Clone)]
pub struct Team {
    name: String,
    score: i32,
}

#[derive(Default)]
struct SetupState {
    teams: Vec<Team>,
}

struct QuizzState {
    quizz: Quizz,
}

impl QuizzState {
    pub fn new(quizz: Quizz) -> QuizzState {
        QuizzState { quizz }
    }
}

pub struct Game {
    current_phase: Phase,
    output_pipe: Arc<RwLock<OutputPipe>>,
}

impl Game {
    pub fn new(output_pipe: OutputPipe) -> Game {
        Game {
            current_phase: Phase::Setup(Default::default()),
            output_pipe: Arc::new(RwLock::new(output_pipe)),
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        match &mut self.current_phase {
            Phase::Setup(_) => (),
            Phase::Quizz(quizz_state) => quizz_state.quizz.tick(dt),
        };
    }

    pub fn begin(&mut self, quizz_path: &Path) -> Result<()> {
        match &self.current_phase {
            Phase::Setup(state) => {
                let definition = QuizzDefinition::open(quizz_path)?;
                let quizz = Quizz::new(definition, state.teams.clone(), self.output_pipe.clone());
                self.current_phase = Phase::Quizz(QuizzState::new(quizz));
                Ok(())
            }
            _ => Err(anyhow!("Cannot call begin outside of setup phase")),
        }
    }

    pub fn guess(&mut self, guess: &str) -> Result<()> {
        match &mut self.current_phase {
            Phase::Setup(_) => Err(anyhow!("Cannot submit answers during setup phase")),
            Phase::Quizz(state) => {
                state.quizz.guess(guess)?;
                Ok(())
            }
        }
    }
}
