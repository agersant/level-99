use anyhow::*;
use parking_lot::RwLock;
use serenity::model::id::UserId;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub mod pool;
mod quizz;

use crate::game::quizz::definition::QuizzDefinition;
use crate::game::quizz::Quizz;
use crate::output::OutputPipe;

#[derive(Debug)]
enum Phase {
    Startup,
    Setup(SetupState),
    Quizz(Quizz),
}

#[derive(Clone, Debug, PartialEq)]
pub enum TeamId {
    TeamName(String),
}

#[derive(Clone, Debug)]
pub struct Team {
    id: TeamId,
    players: HashSet<UserId>,
    score: i32,
}

impl Team {
    pub fn new(id: TeamId) -> Self {
        Team {
            id,
            score: 0,
            players: HashSet::new(),
        }
    }

    pub fn get_display_name(&self) -> &str {
        match &self.id {
            TeamId::TeamName(name) => &name,
        }
    }

    pub fn get_players(&self) -> &HashSet<UserId> {
        &self.players
    }

    pub fn update_score(&mut self, delta: i32) {
        self.score += delta;
    }
}

#[derive(Debug, Default)]
struct SetupState {
    teams: Vec<Team>,
}

impl SetupState {
    pub fn new(teams: Vec<Team>) -> Self {
        SetupState { teams }
    }
}

pub struct Game {
    current_phase: Phase,
    output_pipe: Arc<RwLock<OutputPipe>>,
}

impl Game {
    pub fn new(output_pipe: OutputPipe) -> Game {
        let mut game = Game {
            current_phase: Phase::Startup,
            output_pipe: Arc::new(RwLock::new(output_pipe)),
        };
        game.set_current_phase(Phase::Setup(Default::default()));
        game
    }

    fn set_current_phase(&mut self, phase: Phase) {
        println!("Entering game phase: {:?}", phase);
        self.current_phase = phase;
    }

    pub fn tick(&mut self, dt: Duration) {
        match &mut self.current_phase {
            Phase::Startup | Phase::Setup(_) => (),
            Phase::Quizz(quizz) => {
                quizz.tick(dt);
                if quizz.is_over() {
                    let state = SetupState::new(quizz.get_teams().clone());
                    self.set_current_phase(Phase::Setup(state));
                }
            }
        };
    }

    pub fn begin(&mut self, quizz_path: &Path) -> Result<()> {
        match &self.current_phase {
            Phase::Setup(state) => {
                let definition = QuizzDefinition::open(quizz_path)?;
                let quizz = Quizz::new(definition, state.teams.clone(), self.output_pipe.clone());
                self.set_current_phase(Phase::Quizz(quizz));
                Ok(())
            }
            _ => Err(anyhow!("Cannot call begin outside of setup phase")),
        }
    }

    pub fn guess(&mut self, player: UserId, guess: &str) -> Result<()> {
        let team_id = self
            .get_player_team(player)
            .context("Player is not on a team")?
            .id
            .clone();

        match &mut self.current_phase {
            Phase::Quizz(quizz) => {
                quizz.guess(&team_id, guess)?;
                Ok(())
            }
            _ => Err(anyhow!("Cannot submit answers during setup phase")),
        }
    }

    pub fn join_team(&mut self, player: UserId, team_name: &str) -> Result<()> {
        match &mut self.current_phase {
            Phase::Setup(state) => {
                // Remove player from existing team
                for team in state.teams.iter_mut() {
                    team.players.remove(&player);
                }

                // Put player on his desired team
                let team_id = TeamId::TeamName(team_name.into());
                let mut team = state.teams.iter_mut().find(|team| team.id == team_id);
                if team.is_none() {
                    let new_team = Team::new(team_id);
                    state.teams.push(new_team);
                    team = Some(state.teams.iter_mut().last().expect("Team not found"));
                }
                if let Some(team) = team {
                    team.players.insert(player);
                }

                // Remove empty teams
                state.teams.retain(|t| !t.players.is_empty());

                Ok(())
            }
            _ => Err(anyhow!("Cannot join a team outside of setup phase")),
        }
    }

    fn get_player_team(&self, player: UserId) -> Option<&Team> {
        let teams = self.get_teams();
        teams.iter().find(|t| t.players.contains(&player))
    }

    pub fn get_teams(&self) -> &Vec<Team> {
        match &self.current_phase {
            Phase::Setup(state) => &state.teams,
            Phase::Quizz(quizz) => quizz.get_teams(),
            Phase::Startup => unreachable!(), // TODO inelegant
        }
    }
}
