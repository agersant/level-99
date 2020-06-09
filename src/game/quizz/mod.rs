use anyhow::*;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

use self::definition::*;
use self::phase::*;
use self::settings::*;
use crate::game::Team;
use crate::output::OutputPipe;

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
            let mut output_pipe = self.output_pipe.write();
            let state = self.current_phase.get_state();
            state.tick(&mut output_pipe, dt)
        };
        self.process_transition(transition);
    }

    pub fn guess(&mut self, guess: &str) -> Result<()> {
        match &mut self.current_phase {
            Phase::Question(question_state) => {
                let guessed_correctly = {
                    let mut output_pipe = self.output_pipe.write();
                    question_state.guess(guess, &mut output_pipe)
                };
                if guessed_correctly {
                    self.process_transition(Some(Transition::ToCooldownPhase))
                }
                Ok(())
            }
            _ => Err(anyhow!("There is no active question")),
        }
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
