use anyhow::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Duration;

use crate::game::quiz::assets::*;
use crate::game::quiz::definition::Question;
use crate::game::quiz::State;
use crate::game::{TeamId, TeamsHandle};
use crate::output::{GameOutput, Message, Recipient};

#[derive(Clone, Debug)]
pub struct WagerState<O> {
    pub question: Question,
    time_elapsed: Duration,
    time_limit: Duration,
    teams: TeamsHandle,
    output: O,
    pub participants: HashSet<TeamId>,
    pub wagers: HashMap<TeamId, u32>,
    max_question_score_value: u32,
}

impl<O: GameOutput> WagerState<O> {
    pub fn new(
        question: Question,
        duration: Duration,
        teams: TeamsHandle,
        output: O,
        participants: HashSet<TeamId>,
        max_question_score_value: u32,
    ) -> Self {
        WagerState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
            teams,
            output,
            participants,
            wagers: HashMap::new(),
            max_question_score_value,
        }
    }

    pub fn wager(&mut self, team_id: &TeamId, amount: u32) -> Result<()> {
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

    fn print_time_remaining(&self, before: &Option<Duration>, after: &Option<Duration>) {
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
}

impl<O: GameOutput> State for WagerState<O> {
    fn on_tick(&mut self, dt: Duration) {
        let time_remaining_before = self.time_limit.checked_sub(self.time_elapsed);
        self.time_elapsed += dt;
        let time_remaining_after = self.time_limit.checked_sub(self.time_elapsed);

        if !self.did_every_team_wager() {
            self.print_time_remaining(&time_remaining_before, &time_remaining_after);
        }
    }

    fn on_begin(&mut self) {
        self.output.play_file_audio(Path::new(SFX_CHALLENGE)).ok();
        self.output.say(
            &Recipient::AllTeams,
            &Message::WagerBegins(self.question.category.clone()),
        );
        for team in self.teams.read().iter() {
            if self.participants.contains(&team.id) {
                let wager_cap = self.get_wager_cap(&team.id);
                self.output.say(
                    &Recipient::Team(team.id.clone()),
                    &Message::WagerRules(self.question.score_value, wager_cap),
                );
            } else {
                self.output
                    .say(&Recipient::Team(team.id.clone()), &Message::WagerWait);
            }
        }
    }

    fn on_end(&mut self) {
        let wagers = self
            .participants
            .iter()
            .map(|team_id| {
                let amount = self
                    .wagers
                    .get(team_id)
                    .copied()
                    .unwrap_or(self.question.score_value);
                (team_id.clone(), amount)
            })
            .collect();
        self.output
            .say(&Recipient::AllTeams, &Message::WagerResults(wagers));
    }

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_limit || self.wagers.len() == self.participants.len()
    }
}
