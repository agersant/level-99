use anyhow::*;
use std::collections::HashSet;
use std::time::Duration;

use crate::game::quizz::definition::Question;
use crate::game::quizz::{State, Transition};
use crate::game::TeamId;
use crate::output::{OutputPipe, Payload};

#[derive(Debug)]
pub struct QuestionState {
    question: Question,
    time_elapsed: Duration,
    time_limit: Duration,
    teams_with_guesses: HashSet<TeamId>,
}

impl QuestionState {
    pub fn new(question: Question, duration: Duration) -> Self {
        QuestionState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
            teams_with_guesses: HashSet::new(),
        }
    }

    pub fn guess(
        &mut self,
        team_id: &TeamId,
        guess: &str,
        output_pipe: &mut OutputPipe,
    ) -> Result<bool> {
        if self.teams_with_guesses.contains(team_id) {
            return Err(anyhow!("Team already made a guess"));
        }
        self.teams_with_guesses.insert(team_id.clone());
        let guessed_correctly = self.question.is_guess_correct(guess);
        if guessed_correctly {
            output_pipe.push(Payload::Text("Correct!".into()));
        } else {
            output_pipe.push(Payload::Text("WRONG!".into()));
        }
        Ok(guessed_correctly)
    }

    pub fn get_question(&self) -> &Question {
        &self.question
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_limit
    }
}

impl State for QuestionState {
    fn tick(&mut self, output_pipe: &mut OutputPipe, dt: Duration) -> Option<Transition> {
        self.time_elapsed += dt;
        if !self.is_over() {
            None
        } else {
            output_pipe.push(Payload::Text(
                "Time's up! No one guessed the answer.".into(),
            ));
            Some(Transition::ToCooldownPhase)
        }
    }

    fn begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::Text("Time for a question!".into()));
        output_pipe.push(Payload::Audio(self.question.url.clone()));
    }

    fn end(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::StopAudio);
    }
}
