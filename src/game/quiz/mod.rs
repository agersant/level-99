use anyhow::*;
use std::collections::HashSet;
use std::time::Duration;

use self::definition::*;
use self::phase::*;
use self::settings::*;
use crate::game::{TeamId, TeamsHandle};
use crate::output::{GameOutput, Message, Recipient};

pub mod assets;
pub mod definition;
mod phase;
mod settings;

trait State {
    fn on_begin(&mut self);
    fn on_tick(&mut self, dt: Duration);
    fn on_end(&mut self);
    fn is_over(&self) -> bool;
}

enum Phase<O: GameOutput> {
    Startup(StartupState<O>),
    Cooldown(CooldownState),
    Vote(VoteState<O>),
    Wager(WagerState<O>),
    Question(QuestionState<O>),
    Results(ResultsState<O>),
}

impl<O: GameOutput> Phase<O> {
    fn get_state(&mut self) -> &mut dyn State {
        match self {
            Phase::Startup(s) => s,
            Phase::Cooldown(s) => s,
            Phase::Vote(s) => s,
            Phase::Wager(s) => s,
            Phase::Question(s) => s,
            Phase::Results(s) => s,
        }
    }
}

pub struct Quiz<O: GameOutput> {
    pub teams: TeamsHandle,
    abort: bool,
    settings: Settings,
    current_phase: Phase<O>,
    initiative: Option<TeamId>,
    remaining_questions: HashSet<Question>,
    max_question_score_value: u32,
    output: O,
}

impl<O: GameOutput + Clone> Quiz<O> {
    pub fn new(definition: QuizDefinition, teams: TeamsHandle, output: O) -> Self {
        let settings: Settings = Default::default();
        let questions = definition.get_questions().clone();
        let song_urls = questions.iter().map(|q| q.url.to_owned()).collect();
        let max_question_score_value = questions.iter().map(|q| q.score_value).max().unwrap_or(0);
        let startup_state =
            StartupState::new(settings.startup_duration, &song_urls, output.clone());
        let mut quiz = Quiz {
            abort: false,
            remaining_questions: questions,
            current_phase: Phase::Startup(startup_state.clone()),
            max_question_score_value,
            initiative: None,
            output,
            settings,
            teams,
        };
        quiz.set_current_phase(Phase::Startup(startup_state));
        quiz
    }

    pub fn is_over(&self) -> bool {
        if self.abort {
            return true;
        }
        match self.current_phase {
            Phase::Results(_) => true,
            _ => false,
        }
    }

    fn set_current_phase(&mut self, phase: Phase<O>) {
        let state = self.current_phase.get_state();
        state.on_end();
        self.current_phase = phase;
        let state = self.current_phase.get_state();
        state.on_begin();
    }

    pub fn tick(&mut self, dt: Duration) {
        let state = self.current_phase.get_state();
        state.on_tick(dt);
        if state.is_over() {
            self.advance();
        }
    }

    pub fn guess(&mut self, team_id: &TeamId, guess: &str) -> Result<()> {
        match &mut self.current_phase {
            Phase::Question(question_state) => {
                let guess_result = question_state.guess(team_id, guess)?;
                if guess_result.is_first_correct {
                    self.initiative = Some(team_id.clone());
                }
                Ok(())
            }
            _ => Err(anyhow!("There is no active question")),
        }
    }

    pub fn wager(&mut self, team_id: &TeamId, amount: u32) -> Result<()> {
        match &mut self.current_phase {
            Phase::Wager(wager_state) => {
                wager_state.wager(team_id, amount)?;
                Ok(())
            }
            _ => Err(anyhow!("This is not the time to wager")),
        }
    }

    pub fn abort(&mut self) {
        self.abort = true;
    }

    pub fn skip_phase(&mut self) {
        self.advance();
    }

    fn advance(&mut self) {
        match &self.current_phase {
            Phase::Startup(s) => {
                if s.preload_succeeded() {
                    self.begin_vote();
                } else {
                    self.output
                        .say(&Recipient::AllTeams, &Message::PreloadFailed);
                    self.abort = true;
                }
            }
            Phase::Vote(_s) => {
                self.initiate_question();
            }
            Phase::Wager(s) => {
                let state = QuestionState::new(
                    s.question.clone(),
                    self.settings.question_duration,
                    self.teams.clone(),
                    self.output.clone(),
                    s.participants.clone(),
                    Some(s.wager_amounts.clone()),
                );
                self.set_current_phase(Phase::Question(state));
            }
            Phase::Question(_s) => {
                let state = CooldownState::new(self.settings.cooldown_duration);
                self.set_current_phase(Phase::Cooldown(state));
            }
            Phase::Cooldown(_s) => {
                let remaining_categories: HashSet<&str> = self
                    .remaining_questions
                    .iter()
                    .map(|q| q.category.as_str())
                    .collect();
                match remaining_categories.len() {
                    0 => self.set_current_phase(Phase::Results(ResultsState::new(
                        self.teams.clone(),
                        self.output.clone(),
                    ))),
                    1 => self.initiate_question(),
                    _ => self.begin_vote(),
                }
            }
            Phase::Results(_s) => (),
        }
    }

    fn initiate_question(&mut self) {
        if let Some(question) = self.select_question() {
            if question.challenge {
                let participants = match &self.initiative {
                    Some(team_id) => {
                        let mut h = HashSet::new();
                        h.insert(team_id.clone());
                        h
                    }
                    None => self.teams.read().iter().map(|t| t.id.clone()).collect(),
                };
                let state = WagerState::new(
                    question,
                    self.settings.wager_duration,
                    self.teams.clone(),
                    self.output.clone(),
                    participants,
                    self.max_question_score_value,
                );
                self.set_current_phase(Phase::Wager(state));
            } else {
                let participants = self.teams.read().iter().map(|t| t.id.clone()).collect();
                let state = QuestionState::new(
                    question,
                    self.settings.question_duration,
                    self.teams.clone(),
                    self.output.clone(),
                    participants,
                    None,
                );
                self.set_current_phase(Phase::Question(state));
            }
        } else {
            self.set_current_phase(Phase::Results(ResultsState::new(
                self.teams.clone(),
                self.output.clone(),
            )));
        }
    }

    fn begin_vote(&mut self) {
        let state = VoteState::new(
            self.settings.vote_duration,
            &self.remaining_questions,
            self.initiative.clone(),
            self.teams.clone(),
            self.output.clone(),
            self.settings.max_vote_options,
        );
        self.set_current_phase(Phase::Vote(state));
    }

    fn select_question(&mut self) -> Option<Question> {
        if let Phase::Vote(vote_state) = &self.current_phase {
            if let Ok(question) = vote_state.compute_vote_result() {
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
