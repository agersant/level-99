use anyhow::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

use self::definition::*;
use self::phase::*;
use self::settings::*;
use crate::game::{Team, TeamId};
use crate::output::{OutputPipe, Payload};

pub mod definition;
mod phase;
mod settings;

trait State {
    fn begin(&mut self, output_pipe: &mut OutputPipe);
    fn tick(&mut self, output_pipe: &mut OutputPipe, dt: Duration) -> Option<Transition>;
    fn end(&mut self, output_pipe: &mut OutputPipe);
}

#[derive(Debug)]
enum Phase {
    Startup(StartupState),
    Cooldown(CooldownState),
    Vote(VoteState),
    Question(QuestionState),
    Results(ResultsState),
}

enum Transition {
    ToCooldownPhase,
    ToVotePhase,
    ToQuestionPhase,
    ToResultsPhase,
}

impl Phase {
    fn get_state(&mut self) -> &mut dyn State {
        match self {
            Phase::Startup(s) => s,
            Phase::Cooldown(s) => s,
            Phase::Vote(s) => s,
            Phase::Question(s) => s,
            Phase::Results(s) => s,
        }
    }
}
#[derive(Debug)]
pub struct Quizz {
    settings: Settings,
    remaining_questions: Vec<Question>,
    current_phase: Phase,
    output_pipe: Arc<RwLock<OutputPipe>>,
    teams: Vec<Team>,
}

impl Quizz {
    pub fn new(
        definition: QuizzDefinition,
        teams: Vec<Team>,
        output_pipe: Arc<RwLock<OutputPipe>>,
    ) -> Quizz {
        let settings: Settings = Default::default();
        let mut quizz = Quizz {
            remaining_questions: definition.get_questions().clone(),
            current_phase: Phase::Startup(StartupState::new()),
            output_pipe,
            settings,
            teams,
        };
        quizz.set_current_phase(Phase::Cooldown(CooldownState::new(
            quizz.settings.cooldown_duration,
        )));
        quizz
    }

    pub fn is_over(&self) -> bool {
        match self.current_phase {
            Phase::Results(_) => true,
            _ => false,
        }
    }

    pub fn get_teams(&self) -> &Vec<Team> {
        &self.teams
    }

    fn get_team_mut(&mut self, id: &TeamId) -> Option<&mut Team> {
        self.teams.iter_mut().find(|t| t.id == *id)
    }

    fn set_current_phase(&mut self, phase: Phase) {
        let mut output_pipe = self.output_pipe.write();

        let state = self.current_phase.get_state();
        state.end(&mut output_pipe);

        println!("Entering quizz phase: {:?}", &phase);
        self.current_phase = phase;

        let state = self.current_phase.get_state();
        state.begin(&mut output_pipe);
    }

    pub fn tick(&mut self, dt: Duration) {
        let transition = {
            let state = self.current_phase.get_state();
            state.tick(&mut self.output_pipe.write(), dt)
        };
        self.process_transition(transition);
    }

    pub fn guess(&mut self, team_id: &TeamId, guess: &str) -> Result<()> {
        let transition = match &mut self.current_phase {
            Phase::Question(question_state) => {
                let guessed_correctly =
                    question_state.guess(team_id, guess, &mut self.output_pipe.write())?;

                let score_value = question_state.get_question().score_value as i32;
                let team = self.get_team_mut(team_id).context("Team not found")?;
                let team_display_name = team.get_display_name().to_owned();

                if guessed_correctly {
                    team.update_score(score_value);
                    self.broadcast(Payload::Text(format!(
                        "Team {} earns {} points!",
                        team_display_name, score_value
                    )));
                    Some(Transition::ToCooldownPhase)
                } else {
                    team.update_score(-score_value);
                    self.broadcast(Payload::Text(format!(
                        "Team {} loses {} points. Womp womp 📯",
                        team_display_name, score_value
                    )));
                    None
                }
            }
            _ => return Err(anyhow!("There is no active question")),
        };

        self.process_transition(transition);
        Ok(())
    }

    fn broadcast(&self, payload: Payload) {
        let mut output_pipe = self.output_pipe.write();
        output_pipe.push(payload);
    }

    fn process_transition(&mut self, transition: Option<Transition>) {
        match transition {
            Some(Transition::ToCooldownPhase) => {
                let state = CooldownState::new(self.settings.cooldown_duration);
                self.set_current_phase(Phase::Cooldown(state));
            }
            Some(Transition::ToVotePhase) => {
                let state = VoteState::new(self.settings.vote_duration);
                self.set_current_phase(Phase::Vote(state));
            }
            Some(Transition::ToQuestionPhase) => {
                match self.select_question() {
                    None => self.process_transition(Some(Transition::ToResultsPhase)),
                    Some(question) => {
                        let state = QuestionState::new(question, self.settings.question_duration);
                        self.set_current_phase(Phase::Question(state));
                    }
                };
            }
            Some(Transition::ToResultsPhase) => {
                let state = ResultsState::new();
                self.set_current_phase(Phase::Results(state));
            }
            None => (),
        }
    }

    fn select_question(&mut self) -> Option<Question> {
        if self.remaining_questions.is_empty() {
            return None;
        }
        Some(self.remaining_questions.swap_remove(0))
    }
}
