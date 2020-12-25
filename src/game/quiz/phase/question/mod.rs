use anyhow::*;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use crate::game::quiz::assets::*;
use crate::game::quiz::definition::Question;
use crate::game::quiz::State;
use crate::game::{TeamId, TeamsHandle};
use crate::output::{AudioHandle, GameOutput, Message, Recipient};
use crate::preload;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct GuessResult {
    pub guess: String,
    pub score_delta: i32,
    pub is_correct: bool,
    pub is_first_correct: bool,
}

pub struct QuestionState<O: GameOutput> {
    question: Question,
    time_elapsed: Duration,
    default_time_limit: Duration,
    guesses: HashMap<TeamId, GuessResult>,
    teams: TeamsHandle,
    participants: HashSet<TeamId>,
    wagers: Option<HashMap<TeamId, u32>>,
    countdown_audio: Option<O::Audio>,
    song_audio: Option<O::Audio>,
    output: O,
}

impl<O: GameOutput> QuestionState<O> {
    pub fn new(
        question: Question,
        duration: Duration,
        teams: TeamsHandle,
        output: O,
        participants: HashSet<TeamId>,
        wagers: Option<HashMap<TeamId, u32>>,
    ) -> Self {
        QuestionState {
            question,
            time_elapsed: Duration::default(),
            default_time_limit: duration,
            guesses: HashMap::new(),
            teams,
            participants,
            wagers,
            countdown_audio: None,
            song_audio: None,
            output,
        }
    }

    pub fn guess(&mut self, team_id: &TeamId, guess: &str) -> Result<GuessResult> {
        if self.guesses.contains_key(team_id) {
            return Err(anyhow!("Team already made a guess"));
        }

        if !self.participants.contains(team_id) {
            return Err(anyhow!("Your team is not allowed to answer this question"));
        }

        let is_correct = self.question.is_guess_correct(guess);
        let score_delta = self.compute_score_delta(team_id, is_correct);
        let is_first_correct = is_correct && !self.was_correctly_guessed();
        let guess_result = GuessResult {
            guess: guess.into(),
            is_correct,
            score_delta,
            is_first_correct,
        };
        self.guesses.insert(team_id.clone(), guess_result.clone());

        self.teams
            .write()
            .iter_mut()
            .find(|t| t.id == *team_id)
            .context("Team not found")?
            .update_score(guess_result.score_delta);

        if guess_result.is_correct {
            self.output.play_file_audio(Path::new(SFX_CORRECT)).ok();
            self.output.say(
                &Recipient::AllTeams,
                &Message::GuessCorrect(team_id.clone(), guess_result.score_delta),
            );
        } else {
            self.output.play_file_audio(Path::new(SFX_INCORRECT)).ok();
            self.output.say(
                &Recipient::AllTeams,
                &Message::GuessIncorrect(team_id.clone(), guess_result.score_delta.abs()),
            );
        }

        if self.did_every_team_submit_a_guess() {
            self.output.say(
                &Recipient::AllTeams,
                &Message::AnswerReveal(self.question.clone()),
            );
            self.reveal_guesses();
        }

        Ok(guess_result)
    }

    fn was_correctly_guessed(&self) -> bool {
        self.guesses.iter().any(|(_t, g)| g.is_correct)
    }

    fn did_every_team_submit_a_guess(&self) -> bool {
        self.guesses.len() == self.participants.len()
    }

    fn compute_score_value(&self, team_id: &TeamId) -> i32 {
        let score_value = self
            .wagers
            .as_ref()
            .and_then(|w| w.get(team_id).copied())
            .unwrap_or(self.question.score_value) as i32;
        let is_first_guess = self.guesses.is_empty();
        if is_first_guess || self.wagers.is_some() {
            score_value
        } else {
            score_value / 2
        }
    }

    fn compute_score_delta(&self, team_id: &TeamId, correct: bool) -> i32 {
        let score_value = self.compute_score_value(team_id);
        let correctness_multiplier = if correct { 1 } else { -1 };
        score_value * correctness_multiplier
    }

    fn reveal_guesses(&mut self) {
        if self.guesses.is_empty() {
            return;
        }
        let guesses = self
            .guesses
            .iter()
            .map(|(team_id, guess_result)| (team_id.clone(), guess_result.guess.clone()))
            .collect();
        self.output
            .say(&Recipient::AllTeams, &Message::GuessesReveal(guesses));
    }

    fn print_scores(&mut self) {
        let mut teams = self.teams.read().clone();
        teams.sort_by_key(|t| Reverse(t.score));
        let teams = teams.iter().map(|t| (t.id.clone(), t.score)).collect();
        self.output
            .say(&Recipient::AllTeams, &Message::ScoresRecap(teams));
    }

    fn print_time_remaining(&mut self, before: &Option<Duration>, after: &Option<Duration>) {
        match (before, after) {
            (Some(before), Some(after)) => {
                let seconds_10 = Duration::from_secs(10);
                let seconds_30 = Duration::from_secs(30);
                let threshold_10 = *before > seconds_10 && *after <= seconds_10;
                let threshold_30 = *before > seconds_30 && *after <= seconds_30;
                if threshold_10 {
                    self.output.say(
                        &Recipient::AllTeams,
                        &Message::TimeRemaining(Duration::from_secs(10)),
                    );
                } else if threshold_30 {
                    self.output.say(
                        &Recipient::AllTeams,
                        &Message::TimeRemaining(Duration::from_secs(30)),
                    );
                }
            }
            _ => (),
        };
    }

    fn get_time_limit(&self) -> Duration {
        self.question.duration.unwrap_or(self.default_time_limit)
    }
}

impl<O: GameOutput> State for QuestionState<O> {
    fn on_tick(&mut self, dt: Duration) {
        let time_limit = self.get_time_limit();
        let time_remaining_before = time_limit.checked_sub(self.time_elapsed);
        self.time_elapsed += dt;
        let time_remaining_after = time_limit.checked_sub(self.time_elapsed);

        if !self.did_every_team_submit_a_guess() {
            self.print_time_remaining(&time_remaining_before, &time_remaining_after);
        }

        let should_start_song = match (&self.countdown_audio, &self.song_audio) {
            (_, Some(_)) => false,
            (Some(a), None) => a.is_finished(),
            (None, None) => true,
        };
        if should_start_song {
            if let Some(cache_entry) = preload::retrieve_song(&self.question.url) {
                self.song_audio = self.output.play_file_audio(&cache_entry.path).ok();
            } else {
                self.song_audio = self
                    .output
                    .play_youtube_audio(self.question.url.clone())
                    .ok();
            }
        }
    }

    fn on_begin(&mut self) {
        self.countdown_audio = self.output.play_file_audio(Path::new(SFX_QUESTION)).ok();
        if self.wagers.is_some() {
            for team in self.teams.read().iter() {
                if self.participants.contains(&team.id) {
                    self.output.say(
                        &Recipient::Team(team.id.clone()),
                        &Message::ChallengeSongBegins(self.question.category.clone()),
                    );
                }
            }
        } else {
            self.output.say(
                &Recipient::AllTeams,
                &Message::QuestionBegins(self.question.clone()),
            );
        }
    }

    fn on_end(&mut self) {
        self.output.stop_audio().ok();

        if !self.did_every_team_submit_a_guess() {
            // Reveal answer
            self.output.play_file_audio(Path::new(SFX_TIME)).ok();
            self.output.say(
                &Recipient::AllTeams,
                &Message::TimeUp(self.question.clone()),
            );

            // Show all guesses
            self.reveal_guesses();

            // Deduct points for unanswered wager
            if self.wagers.is_some() {
                for team_id in &self.participants {
                    if self.guesses.get(team_id).is_none() {
                        let score_value = self.compute_score_value(team_id);
                        if let Some(team) = self.teams.write().iter_mut().find(|t| t.id == *team_id)
                        {
                            team.update_score(-score_value);
                        }
                        self.output.say(
                            &Recipient::AllTeams,
                            &Message::ChallengeSongTimeUp(team_id.clone(), score_value),
                        );
                    }
                }
            }
        }

        self.print_scores();
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.get_time_limit()
    }
}
