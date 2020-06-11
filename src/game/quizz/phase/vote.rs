use anyhow::*;
use itertools::Itertools;
use rand::seq::SliceRandom;
use serenity::model::id::{ChannelId, MessageId};
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::time::Duration;

use crate::game::quizz::definition::Question;
use crate::game::quizz::State;
use crate::game::{TeamId, TeamsHandle};
use crate::output::{OutputPipe, Recipient};

const VOTE_REACTIONS: &'static [&'static str] =
    &["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣", "🔟"];

#[derive(Debug)]
pub struct VoteState {
    time_elapsed: Duration,
    time_to_wait: Duration,
    vote_options: Vec<Question>,
    voting_team: Option<TeamId>,
    teams: TeamsHandle,
    vote_message_ids: Option<HashMap<TeamId, Result<(ChannelId, MessageId)>>>,
}

impl VoteState {
    pub fn new(
        duration: Duration,
        remaining_questions: &HashSet<Question>,
        voting_team: Option<TeamId>,
        teams: TeamsHandle,
        max_vote_options: usize,
    ) -> Self {
        let state = VoteState {
            time_elapsed: Duration::default(),
            time_to_wait: duration,
            vote_options: VoteState::select_vote_options(remaining_questions, max_vote_options),
            voting_team,
            teams,
            vote_message_ids: None,
        };
        state
    }

    fn select_vote_options(
        remaining_questions: &HashSet<Question>,
        max_vote_options: usize,
    ) -> Vec<Question> {
        let max_vote_options = max_vote_options.min(VOTE_REACTIONS.len());

        let lowest_value_question_per_category: Vec<&Question> = remaining_questions
            .iter()
            .map(|q| (q.category.clone(), q))
            .into_group_map()
            .into_iter()
            .map(|(_c, questions)| {
                questions
                    .into_iter()
                    .min_by_key(|q| q.score_value)
                    .expect("Empty category in group map")
            })
            .collect();

        let mut rng = &mut rand::thread_rng();
        lowest_value_question_per_category
            .choose_multiple(&mut rng, max_vote_options)
            .sorted_by_key(|q| &q.category)
            .cloned()
            .cloned()
            .collect()
    }

    pub fn compute_vote_result(&self, output_pipe: &mut OutputPipe) -> Result<Question> {
        let message_ids = self.vote_message_ids.as_ref().context("No vote message")?;
        let mut vote_counts = HashMap::new();

        for (_team_id, ids) in message_ids {
            if let Ok((channel_id, message_id)) = ids {
                for (index, question) in self.vote_options.iter().enumerate() {
                    vote_counts.insert(question, 0);
                    let count = vote_counts.get_mut(question).expect("Question not found");

                    let reaction = VOTE_REACTIONS[index].into();
                    let players = output_pipe
                        .read_reactions(*channel_id, *message_id, reaction)
                        .context("Could not read vote reactions")?;

                    for player in &players {
                        let is_valid_vote = match &self.voting_team {
                            Some(team_id) => self
                                .teams
                                .read()
                                .iter()
                                .find(|t| &t.id == team_id)
                                .and_then(|t| Some(t.players.contains(player)))
                                .unwrap_or(false),
                            None => true,
                        };
                        if is_valid_vote {
                            *count += 1;
                        }
                    }
                }
            }
        }

        let vote_counts = vote_counts.into_iter().collect_vec();
        let max_votes = vote_counts
            .iter()
            .max_by_key(|(_q, n)| n)
            .and_then(|(_q, n)| Some(*n))
            .context("Could not find questions with most votes")?;

        let questions_with_max_votes = vote_counts
            .iter()
            .filter_map(|(q, n)| if *n < max_votes { None } else { Some(*q) })
            .collect_vec();

        let mut rng = &mut rand::thread_rng();
        let chosen_question = questions_with_max_votes
            .choose(&mut rng)
            .context("Could not randomly select question")?
            .deref();

        Ok(chosen_question.clone())
    }
}

impl State for VoteState {
    fn on_begin(&mut self, output_pipe: &mut OutputPipe) {
        match &self.voting_team {
            Some(team_id) => match team_id {
                TeamId::TeamName(team_name) => {
                    output_pipe.say(
                        &Recipient::AllTeamsExcept(team_id.clone()),
                        &format!(
                            "**{}** is choosing a category for the next question.",
                            team_name
                        ),
                    );
                }
            },
            _ => (),
        };

        let mut poll_message: String =
            "Vote for the next question by reacting to this message! 🗳️\n".to_owned();
        let mut reactions = Vec::new();
        for (index, question) in self.vote_options.iter().enumerate() {
            poll_message.push_str(&format!(
                "\n{} **{}** {}pts",
                VOTE_REACTIONS[index], question.category, question.score_value
            ));
            reactions.push(VOTE_REACTIONS[index].into());
        }
        let recipient = match &self.voting_team {
            None => Recipient::AllTeams,
            Some(team_id) => Recipient::Team(team_id.clone()),
        };
        self.vote_message_ids =
            Some(output_pipe.say_with_reactions(&recipient, &poll_message, &reactions));
    }

    fn on_tick(&mut self, _output_pipe: &mut OutputPipe, dt: Duration) {
        self.time_elapsed += dt;
    }

    fn on_end(&mut self, _output_pipe: &mut OutputPipe) {}

    fn is_over(&self) -> bool {
        self.time_elapsed >= self.time_to_wait
    }
}
