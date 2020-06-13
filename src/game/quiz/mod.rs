use anyhow::*;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use self::definition::*;
use self::phase::*;
use self::settings::*;
use crate::game::{TeamId, TeamsHandle};
use crate::output::OutputPipe;

pub mod definition;
mod phase;
mod settings;

trait State {
    fn on_begin(&mut self, output_pipe: &mut OutputPipe);
    fn on_tick(&mut self, output_pipe: &mut OutputPipe, dt: Duration);
    fn on_end(&mut self, output_pipe: &mut OutputPipe);
    fn is_over(&self) -> bool;
}

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
pub struct Quiz {
    pub teams: TeamsHandle,
    settings: Settings,
    current_phase: Phase,
    initiative: Option<TeamId>,
    remaining_questions: HashSet<Question>,
    output_pipe: Arc<RwLock<OutputPipe>>,
}

impl Quiz {
    pub fn new(
        definition: QuizDefinition,
        teams: TeamsHandle,
        output_pipe: Arc<RwLock<OutputPipe>>,
    ) -> Quiz {
        let settings: Settings = Default::default();
        let startup_state =
            StartupState::new(settings.startup_duration, definition.get_questions());
        let mut quiz = Quiz {
            remaining_questions: definition.get_questions().clone(),
            current_phase: Phase::Startup(startup_state.clone()),
            initiative: None,
            output_pipe,
            settings,
            teams,
        };
        quiz.set_current_phase(Phase::Startup(startup_state));
        quiz
    }

    pub fn is_over(&self) -> bool {
        match self.current_phase {
            Phase::Results(_) => true,
            _ => false,
        }
    }

    fn set_current_phase(&mut self, phase: Phase) {
        let mut output_pipe = self.output_pipe.write();

        let state = self.current_phase.get_state();
        state.on_end(&mut output_pipe);

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
                if guess_result.is_first_correct {
                    self.initiative = Some(team_id.clone());
                }
                Ok(())
            }
            _ => Err(anyhow!("There is no active question")),
        }
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
                self.begin_question();
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
                let remaining_categories: HashSet<&str> = self
                    .remaining_questions
                    .iter()
                    .map(|q| q.category.as_str())
                    .collect();
                if remaining_categories.len() > 1 {
                    self.begin_vote();
                } else {
                    self.begin_question();
                }
            }
            Phase::Results(_s) => (),
        }
    }

    fn begin_question(&mut self) {
        if let Some(question) = self.select_question() {
            let state = QuestionState::new(
                question,
                self.settings.question_duration,
                self.teams.clone(),
            );
            self.set_current_phase(Phase::Question(state));
        } else {
            self.set_current_phase(Phase::Results(ResultsState::new()));
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
            let mut output_pipe = self.output_pipe.write();
            if let Ok(question) = vote_state.compute_vote_result(&mut output_pipe) {
                return self.remaining_questions.take(&question);
            }
        }
        let question = self
            .remaining_questions
            .iter()
            .min_by_key(|q| q.score_value)
            .cloned();
        if let Some(question) = question {
            return self.remaining_questions.take(&question);
        }
        None
    }
}
