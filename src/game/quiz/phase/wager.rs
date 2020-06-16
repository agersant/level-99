use anyhow::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use crate::game::quiz::assets::*;
use crate::game::quiz::definition::Question;
use crate::game::quiz::State;
use crate::game::{TeamId, TeamsHandle};
use crate::output::{OutputPipe, Recipient};

#[derive(Clone, Debug)]
pub struct WagerState {
    pub question: Question,
    time_elapsed: Duration,
    time_limit: Duration,
    teams: TeamsHandle,
    pub participants: HashSet<TeamId>,
    pub wagers: HashMap<TeamId, u32>,
    max_question_score_value: u32,
}

impl WagerState {
    pub fn new(
        question: Question,
        duration: Duration,
        teams: TeamsHandle,
        participants: HashSet<TeamId>,
        max_question_score_value: u32,
    ) -> Self {
        WagerState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
            teams,
            participants,
            wagers: HashMap::new(),
            max_question_score_value,
        }
    }

    pub fn wager(
        &mut self,
        team_id: &TeamId,
        amount: u32,
        _output_pipe: &mut OutputPipe,
    ) -> Result<()> {
        if !self.participants.contains(team_id) {
            return Err(anyhow!("Your team is not allowed to wager."));
        }
        let wager_cap = self.get_wager_cap(team_id);
        let amount = amount.min(wager_cap).max(self.question.score_value);
        self.wagers.insert(team_id.clone(), amount);
        Ok(())
    }

    fn get_wager_cap(&self, team_id: &TeamId) -> u32 {
        let team_score = self
            .teams
            .read()
            .iter()
            .find(|t| &t.id == team_id)
            .and_then(|t| Some(t.score.max(0) as u32))
            .unwrap_or(0);
        team_score.max(2 * self.max_question_score_value)
    }

    fn did_every_team_wager(&self) -> bool {
        self.wagers.len() == self.participants.len()
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

impl State for WagerState {
    fn on_tick(&mut self, output_pipe: &mut OutputPipe, dt: Duration) {
        let time_remaining_before = self.time_limit.checked_sub(self.time_elapsed);
        self.time_elapsed += dt;
        let time_remaining_after = self.time_limit.checked_sub(self.time_elapsed);

        if !self.did_every_team_wager() {
            self.print_time_remaining(output_pipe, &time_remaining_before, &time_remaining_after);
        }
    }

    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.play_file_audio(Path::new(SFX_CHALLENGE)).ok();
        output_pipe.say(
            &Recipient::AllTeams,
            &format!(
                "⚠️ A **CHALLENGE** question has appeared in the **{}** category!",
                self.question.category
            ),
        );
        for team in self.teams.read().iter() {
            if self.participants.contains(&team.id) {
                let wager_cap = self.get_wager_cap(&team.id);
                output_pipe.say(
                    &Recipient::Team(team.id.clone()),
                    &format!(
                        "🍀 **Your team must answer this question**. Use the `!wager amount` command to wager between {} and {} points. This is the amount your team will earn or lose from this question.",
                        self.question.score_value, wager_cap
                    ),
                );
            } else {
                output_pipe.say(
                    &Recipient::Team(team.id.clone()),
                    "⏳ Please wait while other teams are responding to the **CHALLENGE** question.",
                );
            }
        }
    }

    fn on_end(&mut self, output_pipe: &mut OutputPipe) {
        let mut message = "".to_owned();
        for team_id in &self.participants {
            let amount = self
                .wagers
                .get(team_id)
                .copied()
                .unwrap_or(self.question.score_value);
            let team_display_name = {
                let teams = self.teams.read();
                teams
                    .iter()
                    .find(|t| t.id == *team_id)
                    .and_then(|t| Some(t.get_display_name().to_owned()))
            };
            if let Some(team_display_name) = team_display_name {
                message.push_str(&format!(
                    "**Team {}** is betting *{} points*!\n",
                    team_display_name, amount
                ));
            }
        }
        output_pipe.say(&Recipient::AllTeams, &message);
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_limit || self.wagers.len() == self.participants.len()
    }
}
