use anyhow::*;
use std::path::Path;

mod definition;
pub mod manager;
mod quizz;
mod settings;

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
    phase: Phase,
}

impl Game {
    pub fn new() -> Game {
        Game {
            phase: Phase::Setup(Default::default()),
        }
    }

    pub fn begin(&mut self, quizz_path: &Path) -> Result<()> {
        match &self.phase {
            Phase::Setup(state) => {
                let quizz = Quizz::load(quizz_path)?;
                self.phase = Phase::Quizz(QuizzState::new(state.teams.clone(), quizz));
                Ok(())
            }
            _ => Err(anyhow!("Cannot call begin outside of setup phase")),
        }
    }
}
