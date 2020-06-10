use anyhow::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::game::quizz::definition::Question;
use crate::game::quizz::{State, Transition};
use crate::game::TeamId;
use crate::output::{OutputPipe, Payload};

#[derive(Clone, Debug)]
pub struct GuessResult {
    pub guess: String,
    pub score_delta: i32,
    pub is_correct: bool,
    pub is_first_correct: bool,
}

#[derive(Debug)]
pub struct QuestionState {
    question: Question,
    time_elapsed: Duration,
    time_limit: Duration,
    guesses: HashMap<TeamId, GuessResult>,
}

impl QuestionState {
    pub fn new(question: Question, duration: Duration) -> Self {
        QuestionState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
            guesses: HashMap::new(),
        }
    }

    pub fn guess(
        &mut self,
        team_id: &TeamId,
        guess: &str,
        _output_pipe: &mut OutputPipe,
    ) -> Result<GuessResult> {
        if self.guesses.contains_key(team_id) {
            return Err(anyhow!("Team already made a guess"));
        }

        let is_correct = self.question.is_guess_correct(guess);
        let score_delta = self.compute_score_delta(is_correct);
        let is_first_correct = is_correct && !self.was_correctly_guessed();
        let guess_result = GuessResult {
            guess: guess.into(),
            is_correct,
            score_delta,
            is_first_correct,
        };
        self.guesses.insert(team_id.clone(), guess_result.clone());
        Ok(guess_result)
    }

    fn was_correctly_guessed(&self) -> bool {
        self.guesses.iter().any(|(_t, g)| g.is_correct)
    }

    fn compute_score_delta(&self, correct: bool) -> i32 {
        let score_value = self.question.score_value as i32;
        let correctness_multiplier = if correct { 1 } else { -1 };
        let already_guessed = self.was_correctly_guessed();
        let delta = score_value * correctness_multiplier;
        if !already_guessed {
            delta
        } else {
            delta / 2
        }
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
            output_pipe.push(Payload::Text(format!(
                "⏰ Time's up! The answer was _{}_:\n{}",
                self.question.answer, self.question.url
            )));
            Some(Transition::ToCooldownPhase)
        }
    }

    fn begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::Text(format!(
            "🎧 Here's a song from the **{}** category for {} points!",
            self.question.category, self.question.score_value
        )));
        output_pipe.push(Payload::Audio(self.question.url.clone()));
    }

    fn end(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::StopAudio);
    }
}
