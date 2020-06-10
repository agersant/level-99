use anyhow::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::Duration;

use crate::game::quizz::definition::Question;
use crate::game::quizz::State;
use crate::game::{TeamId, TeamsHandle};
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
    teams: TeamsHandle,
}

impl QuestionState {
    pub fn new(question: Question, duration: Duration, teams: TeamsHandle) -> Self {
        QuestionState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
            guesses: HashMap::new(),
            teams,
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

    fn print_scores(&self, output_pipe: &mut OutputPipe) {
        let mut teams = self.teams.read().clone();
        teams.sort_by_key(|t| Reverse(t.score));

        let mut recap = "Here are the scores so far:".to_owned();
        for (index, team) in teams.iter().enumerate() {
            let rank = match index {
                0 => "🥇".to_owned(),
                1 => "🥈".to_owned(),
                2 => "🥉".to_owned(),
                _ => format!("#{}", index + 1),
            };
            recap.push_str(&format!(
                "\n{} **Team {}** with {} points",
                rank,
                team.get_display_name(),
                team.score
            ));
        }

        output_pipe.push(Payload::Text(recap));
    }
}

impl State for QuestionState {
    fn on_tick(&mut self, _output_pipe: &mut OutputPipe, dt: Duration) {
        self.time_elapsed += dt;
    }

    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::Text(format!(
            "🎧 Here's a song from the **{}** category for {} points!",
            self.question.category, self.question.score_value
        )));
        output_pipe.push(Payload::Audio(self.question.url.clone()));
    }

    fn on_end(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::Text(format!(
            "⏰ Time's up! The answer was **{}**:\n{}",
            self.question.answer, self.question.url
        )));
        output_pipe.push(Payload::StopAudio);
        self.print_scores(output_pipe);
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_limit
    }
}
