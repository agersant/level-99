use anyhow::*;
use std::path::Path;
use std::time::Duration;

mod definition;
pub mod output;
pub mod pool;
mod quizz;
mod settings;

use crate::game::output::OutputPipe;
use crate::game::quizz::Quizz;

enum Phase {
    Setup(SetupState),
    Quizz(QuizzState),
}

#[derive(Clone)]
struct Team {
    name: String,
    score: i32,
}

#[derive(Default)]
struct SetupState {
    teams: Vec<Team>,
}

struct QuizzState {
    teams: Vec<Team>,
    quizz: Quizz,
}

impl QuizzState {
    pub fn new(teams: Vec<Team>, quizz: Quizz) -> QuizzState {
        QuizzState { teams, quizz }
    }
}

pub struct Game {
    current_phase: Phase,
    output_pipe: OutputPipe,
}

impl Game {
    pub fn new(output_pipe: OutputPipe) -> Game {
        Game {
            current_phase: Phase::Setup(Default::default()),
            output_pipe,
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        match &mut self.current_phase {
            Phase::Setup(_) => (),
            Phase::Quizz(quizz_state) => quizz_state.quizz.tick(dt, &mut self.output_pipe),
        };
    }

    pub fn begin(&mut self, quizz_path: &Path) -> Result<()> {
        match &self.current_phase {
            Phase::Setup(state) => {
                let quizz = Quizz::load(quizz_path)?;
                self.current_phase = Phase::Quizz(QuizzState::new(state.teams.clone(), quizz));
                Ok(())
            }
            _ => Err(anyhow!("Cannot call begin outside of setup phase")),
        }
    }

    pub fn guess(&mut self, guess: &str) -> Result<()> {
        match &mut self.current_phase {
            Phase::Setup(_) => Err(anyhow!("Cannot submit answers during setup phase")),
            Phase::Quizz(state) => state.quizz.guess(guess, &mut self.output_pipe),
        }
    }
}
