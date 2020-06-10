use anyhow::*;
use parking_lot::RwLock;
use serenity::model::id::UserId;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

pub mod pool;
mod quizz;
pub mod team;

use self::quizz::definition::QuizzDefinition;
use self::quizz::Quizz;
use self::team::{Team, TeamId, TeamsHandle};
use crate::output::OutputPipe;

#[derive(Debug)]
enum Phase {
    Startup,
    Setup,
    Quizz(Quizz),
}

pub struct Game {
    current_phase: Phase,
    teams: TeamsHandle,
    output_pipe: Arc<RwLock<OutputPipe>>,
}

impl Game {
    pub fn new(output_pipe: OutputPipe) -> Game {
        let mut game = Game {
            current_phase: Phase::Startup,
            teams: Arc::new(RwLock::new(Vec::new())),
            output_pipe: Arc::new(RwLock::new(output_pipe)),
        };
        game.set_current_phase(Phase::Setup);
        game
    }

    fn set_current_phase(&mut self, phase: Phase) {
        println!("Entering game phase: {:?}", phase);
        self.current_phase = phase;
    }

    pub fn tick(&mut self, dt: Duration) {
        match &mut self.current_phase {
            Phase::Startup | Phase::Setup => (),
            Phase::Quizz(quizz) => {
                quizz.tick(dt);
                if quizz.is_over() {
                    self.set_current_phase(Phase::Setup);
                }
            }
        };
    }

    pub fn begin(&mut self, quizz_path: &Path) -> Result<()> {
        match &self.current_phase {
            Phase::Setup => {
                let definition = QuizzDefinition::open(quizz_path)?;
                let quizz = Quizz::new(definition, self.teams.clone(), self.output_pipe.clone());
                self.set_current_phase(Phase::Quizz(quizz));
                Ok(())
            }
            _ => Err(anyhow!("Cannot call begin outside of setup phase")),
        }
    }

    pub fn skip(&mut self) -> Result<()> {
        match &mut self.current_phase {
            Phase::Quizz(q) => {
                q.skip_phase();
                Ok(())
            }
            _ => Err(anyhow!("There is no quizz in progress")),
        }
    }

    pub fn guess(&mut self, player: UserId, guess: &str) -> Result<()> {
        let team_id = self
            .get_player_team(player)
            .context("Player is not on a team")?;

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
            Phase::Setup => {
                let mut teams = self.teams.write();

                // Remove player from existing team
                for team in teams.iter_mut() {
                    team.players.remove(&player);
                }

                // Put player on his desired team
                let team_id = TeamId::TeamName(team_name.into());
                let mut team = teams.iter_mut().find(|team| team.id == team_id);
                if team.is_none() {
                    let new_team = Team::new(team_id);
                    teams.push(new_team);
                    team = Some(teams.iter_mut().last().expect("Team not found"));
                }
                if let Some(team) = team {
                    team.players.insert(player);
                }

                // Remove empty teams
                teams.retain(|t| !t.players.is_empty());

                Ok(())
            }
            _ => Err(anyhow!("Cannot join a team outside of setup phase")),
        }
    }

    fn get_player_team(&self, player: UserId) -> Option<TeamId> {
        let teams = self.get_teams();
        teams
            .iter()
            .find(|t| t.players.contains(&player))
            .and_then(|t| Some(t.id.clone()))
    }

    pub fn get_teams(&self) -> Vec<Team> {
        self.teams.read().clone()
    }
}
