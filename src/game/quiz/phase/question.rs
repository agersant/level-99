use anyhow::*;
use serenity::voice::LockedAudio;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use crate::game::quiz::assets::*;
use crate::game::quiz::definition::Question;
use crate::game::quiz::State;
use crate::game::{TeamId, TeamsHandle};
use crate::output::{OutputPipe, Recipient};
use crate::preload;

#[derive(Clone, Debug)]
pub struct GuessResult {
    pub guess: String,
    pub score_delta: i32,
    pub is_correct: bool,
    pub is_first_correct: bool,
}

pub struct QuestionState {
    question: Question,
    time_elapsed: Duration,
    time_limit: Duration,
    guesses: HashMap<TeamId, GuessResult>,
    teams: TeamsHandle,
    participants: HashSet<TeamId>,
    wagers: Option<HashMap<TeamId, u32>>,
    countdown_audio: Option<LockedAudio>,
    song_audio: Option<LockedAudio>,
}

impl QuestionState {
    pub fn new(
        question: Question,
        duration: Duration,
        teams: TeamsHandle,
        participants: HashSet<TeamId>,
        wagers: Option<HashMap<TeamId, u32>>,
    ) -> Self {
        QuestionState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
            guesses: HashMap::new(),
            teams,
            participants,
            wagers,
            countdown_audio: None,
            song_audio: None,
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
            self.reveal_guesses(output_pipe);
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

    fn reveal_guesses(&self, output_pipe: &mut OutputPipe) {
        if self.guesses.is_empty() {
            return;
        }
        let teams = self.teams.read();
        let mut message = "This is what everyone guessed:".to_owned();
        for (team_id, guess) in &self.guesses {
            if let Some(team) = teams.iter().find(|t| t.id == *team_id) {
                message.push_str(&format!(
                    "\n- **Team {}**: {}",
                    team.get_display_name(),
                    guess.guess
                ));
            }
        }
        output_pipe.say(&Recipient::AllTeams, &message);
    }

    fn print_scores(&self, output_pipe: &mut OutputPipe) {
        let mut teams = self.teams.read().clone();
        teams.sort_by_key(|t| Reverse(t.score));

        let mut recap = "📈 Here are the scores so far:".to_owned();
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

        let should_start_song = match (&self.countdown_audio, &self.song_audio) {
            (_, Some(_)) => false,
            (Some(a), None) => a.lock().finished,
            (None, None) => true,
        };
        if should_start_song {
            if let Some(cache_entry) = preload::retrieve_song(&self.question.url) {
                self.song_audio = output_pipe.play_file_audio(&cache_entry.path).ok();
            } else {
                self.song_audio = output_pipe
                    .play_youtube_audio(self.question.url.clone())
                    .ok();
            }
        }
    }

    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        self.countdown_audio = output_pipe.play_file_audio(Path::new(SFX_QUESTION)).ok();
        if self.wagers.is_some() {
            for team in self.teams.read().iter() {
                if self.participants.contains(&team.id) {
                    output_pipe.say(
                        &Recipient::Team(team.id.clone()),
                        &format!(
                            "🎧 Here is a song from the **{}** category! Your team **must** guess this one right or you will lose points.",
                            self.question.category
                        ),
                    );
                }
            }
        } else {
            output_pipe.say(
                &Recipient::AllTeams,
                &format!(
                    "🎧 Here is a song from the **{}** category for {} points!",
                    self.question.category, self.question.score_value
                ),
            );
        }
    }

    fn on_end(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.stop_audio().ok();

        if !self.did_every_team_submit_a_guess() {
            // Reveal answer
            output_pipe.play_file_audio(Path::new(SFX_TIME)).ok();
            output_pipe.say(
                &Recipient::AllTeams,
                &format!(
                    "⏰ Time's up! The answer was **{}**:\n{}",
                    self.question.answer, self.question.url
                ),
            );

            // Show all guesses
            self.reveal_guesses(output_pipe);

            // Deduct points for unanswered wager
            if self.wagers.is_some() {
                for team_id in &self.participants {
                    if self.guesses.get(team_id).is_none() {
                        if let Some(team) = self.teams.write().iter_mut().find(|t| t.id == *team_id)
                        {
                            let score_value = self.compute_score_value(team_id);
                            team.update_score(-score_value);
                            output_pipe.say(
                                &Recipient::AllTeams,
                                &format!(
                                    "**Team {}** loses *{} points* for not answering the **CHALLENGE** question!",
                                    team.get_display_name(),
                                    score_value
                                ),
                            );
                        }
                    }
                }
            }
        }

        self.print_scores(output_pipe);
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_limit
    }
}
