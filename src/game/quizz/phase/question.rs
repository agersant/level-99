use anyhow::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::game::quizz::definition::Question;
use crate::game::quizz::State;
use crate::game::{TeamId, TeamsHandle};
use crate::output::{OutputPipe, Recipient};

const SFX_CORRECT: &'static str = "assets/correct.wav";
const SFX_INCORRECT: &'static str = "assets/incorrect.wav";
const SFX_QUESTION: &'static str = "assets/question.wav";
const SFX_TIME: &'static str = "assets/time.wav";
const QUESTION_AUDIO_DELAY: Duration = Duration::from_millis(3_500);

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
    started_question_audio: bool,
}

impl QuestionState {
    pub fn new(question: Question, duration: Duration, teams: TeamsHandle) -> Self {
        QuestionState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
            guesses: HashMap::new(),
            teams,
            started_question_audio: false,
        }
    }

    pub fn guess(
        &mut self,
        team_id: &TeamId,
        guess: &str,
        output_pipe: &mut OutputPipe,
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

        let team_display_name = {
            let mut teams = self.teams.write();
            let team = teams
                .iter_mut()
                .find(|t| t.id == *team_id)
                .context("Team not found")?;
            team.update_score(guess_result.score_delta);
            team.get_display_name().to_owned()
        };

        if guess_result.is_correct {
            output_pipe.play_file_audio(Path::new(SFX_CORRECT)).ok();
            output_pipe.say(
                &Recipient::AllTeams,
                &format!(
                    "✅ **Team {}** guessed correctly and earned {} points!",
                    team_display_name, guess_result.score_delta
                ),
            );
        } else {
            output_pipe.play_file_audio(Path::new(SFX_INCORRECT)).ok();
            output_pipe.say(
                &Recipient::AllTeams,
                &format!(
                    "❌ **Team {}** guessed incorrectly and lost {} points. Womp womp 📯.",
                    team_display_name,
                    guess_result.score_delta.abs()
                ),
            );
        }

        if self.did_every_team_submit_a_guess() {
            output_pipe.say(
                &Recipient::AllTeams,
                &format!(
                    "The answer was **{}**:\n{}",
                    self.question.answer, self.question.url
                ),
            );
        }

        Ok(guess_result)
    }

    fn was_correctly_guessed(&self) -> bool {
        self.guesses.iter().any(|(_t, g)| g.is_correct)
    }

    fn did_every_team_submit_a_guess(&self) -> bool {
        self.guesses.len() == self.teams.read().len()
    }

    fn compute_score_delta(&self, correct: bool) -> i32 {
        let score_value = self.question.score_value as i32;
        let correctness_multiplier = if correct { 1 } else { -1 };
        let is_first_guess = self.guesses.is_empty();
        let delta = score_value * correctness_multiplier;
        if is_first_guess {
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

        output_pipe.say(&Recipient::AllTeams, &recap);
    }

    fn print_time_remaining(
        &self,
        output_pipe: &mut OutputPipe,
        before: &Option<Duration>,
        after: &Option<Duration>,
    ) {
        match (before, after) {
            (Some(before), Some(after)) => {
                let seconds_10 = Duration::from_secs(10);
                let seconds_30 = Duration::from_secs(30);
                let threshold_10 = *before > seconds_10 && *after <= seconds_10;
                let threshold_30 = *before > seconds_30 && *after <= seconds_30;
                if threshold_10 {
                    output_pipe.say(&Recipient::AllTeams, "🕒 Only 10 seconds left!");
                } else if threshold_30 {
                    output_pipe.say(&Recipient::AllTeams, "🕒 Only 30 seconds left!");
                }
            }
            _ => (),
        };
    }
}

impl State for QuestionState {
    fn on_tick(&mut self, output_pipe: &mut OutputPipe, dt: Duration) {
        let time_remaining_before = self.time_limit.checked_sub(self.time_elapsed);
        self.time_elapsed += dt;
        let time_remaining_after = self.time_limit.checked_sub(self.time_elapsed);

        if !self.did_every_team_submit_a_guess() {
            self.print_time_remaining(output_pipe, &time_remaining_before, &time_remaining_after);
        }

        if !self.started_question_audio && self.time_elapsed > QUESTION_AUDIO_DELAY {
            self.started_question_audio = true;
            if let Err(e) = output_pipe.play_youtube_audio(self.question.url.clone()) {
                output_pipe.say(
                    &Recipient::AllTeams,
                    &format!("Oops that didn't actually work: {}", e),
                );
            }
        }
    }

    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.play_file_audio(Path::new(SFX_QUESTION)).ok();
        output_pipe.say(
            &Recipient::AllTeams,
            &format!(
                "🎧 Here is a song from the **{}** category for {} points!",
                self.question.category, self.question.score_value
            ),
        );
    }

    fn on_end(&mut self, output_pipe: &mut OutputPipe) {
        if let Err(e) = output_pipe.stop_audio() {
            output_pipe.say(
                &Recipient::AllTeams,
                &format!("There was a problem stopping the music: {}", e),
            );
        }

        if self.did_every_team_submit_a_guess() {
            output_pipe.say(&Recipient::AllTeams, "Let's move on!");
        } else {
            output_pipe.play_file_audio(Path::new(SFX_TIME)).ok();
            output_pipe.say(
                &Recipient::AllTeams,
                &format!(
                    "⏰ Time's up! The answer was **{}**:\n{}",
                    self.question.answer, self.question.url
                ),
            );
        }

        self.print_scores(output_pipe);
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_limit
    }
}
