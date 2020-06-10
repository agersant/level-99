use anyhow::*;
use parking_lot::RwLock;
use std::collections::HashSet;
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
    fn on_begin(&mut self, output_pipe: &mut OutputPipe);
    fn on_tick(&mut self, output_pipe: &mut OutputPipe, dt: Duration);
    fn on_end(&mut self, output_pipe: &mut OutputPipe);
    fn is_over(&self) -> bool;
}

#[derive(Debug)]
enum Phase {
    Startup(StartupState),
    Cooldown(CooldownState),
    Vote(VoteState),
    Question(QuestionState),
    Results(ResultsState),
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
    teams: Vec<Team>,
    settings: Settings,
    current_phase: Phase,
    initiative: Option<TeamId>,
    remaining_questions: HashSet<Question>,
    output_pipe: Arc<RwLock<OutputPipe>>,
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
            initiative: None,
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
        state.on_end(&mut output_pipe);

        println!("Entering quizz phase: {:?}", &phase);
        self.current_phase = phase;

        let state = self.current_phase.get_state();
        state.on_begin(&mut output_pipe);
    }

    pub fn tick(&mut self, dt: Duration) {
        let state = self.current_phase.get_state();
        state.on_tick(&mut self.output_pipe.write(), dt);
        if state.is_over() {
            self.advance();
        }
    }

    pub fn guess(&mut self, team_id: &TeamId, guess: &str) -> Result<()> {
        match &mut self.current_phase {
            Phase::Question(question_state) => {
                let guess_result =
                    question_state.guess(team_id, guess, &mut self.output_pipe.write())?;

                let team = self.get_team_mut(team_id).context("Team not found")?;
                let team_display_name = team.get_display_name().to_owned();

                team.update_score(guess_result.score_delta);
                if guess_result.is_correct {
                    self.broadcast(Payload::Text(format!(
                        "✅ Team {} guessed correctly and earned {} points!",
                        team_display_name, guess_result.score_delta
                    )));
                } else {
                    self.broadcast(Payload::Text(format!(
                        "❌ Team {} guessed incorrectly and lost {} points. Womp womp 📯.",
                        team_display_name,
                        guess_result.score_delta.abs()
                    )));
                }

                if guess_result.is_first_correct {
                    self.initiative = Some(team_id.clone());
                }

                Ok(())
            }
            _ => Err(anyhow!("There is no active question")),
        }
    }

    fn broadcast(&self, payload: Payload) {
        let mut output_pipe = self.output_pipe.write();
        output_pipe.push(payload);
    }

    pub fn skip_phase(&mut self) {
        self.advance();
    }

    fn advance(&mut self) {
        match &self.current_phase {
            Phase::Startup(_s) => {
                self.begin_vote();
            }
            Phase::Vote(_s) => {
                if let Some(question) = self.select_question() {
                    let state = QuestionState::new(question, self.settings.question_duration);
                    self.set_current_phase(Phase::Question(state));
                } else {
                    self.set_current_phase(Phase::Results(ResultsState::new()));
                }
            }
            Phase::Question(_s) => {
                if self.remaining_questions.is_empty() {
                    self.set_current_phase(Phase::Results(ResultsState::new()));
                } else {
                    let state = CooldownState::new(self.settings.cooldown_duration);
                    self.set_current_phase(Phase::Cooldown(state));
                }
            }
            Phase::Cooldown(_s) => {
                self.begin_vote();
            }
            Phase::Results(_s) => (),
        }
    }

    fn begin_vote(&mut self) {
        let state = VoteState::new(
            self.settings.vote_duration,
            &self.remaining_questions,
            self.initiative.clone(),
            self.teams.clone(),
            self.settings.max_vote_options,
        );
        self.set_current_phase(Phase::Vote(state));
    }

    fn select_question(&mut self) -> Option<Question> {
        if let Phase::Vote(vote_state) = &self.current_phase {
            if let Some(question) = vote_state.get_vote_results() {
                return self.remaining_questions.take(question);
            }
        }
        let question = self.remaining_questions.iter().next().cloned();
        if let Some(question) = question {
            return self.remaining_questions.take(&question);
        }
        None
    }
}
