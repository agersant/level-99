use super::*;

use parking_lot::RwLock;
use serenity::model::id::UserId;
use std::sync::Arc;
use std::time::Duration;

use crate::game::quiz::definition::{Question, RawQuestion};
use crate::game::team::Team;
use crate::output::mock::{Entry, MockGameOutput, TextEntry};

struct Context {
    state: VoteState<MockGameOutput>,
    remaining_questions: HashSet<Question>,
    output: MockGameOutput,
    teams: TeamsHandle,
}

struct ContextBuilder {
    // remaining_questions_count: usize,
    questions: HashSet<Question>,
    voting_team: Option<TeamId>,
    team_ids: HashMap<String, TeamId>,
    max_vote_options: usize,
}

impl ContextBuilder {
    fn new() -> Self {
        ContextBuilder {
            questions: HashSet::new(),
            voting_team: None,
            max_vote_options: 5,
            team_ids: ["red", "blue"]
                .iter()
                .map(|n| (n.to_string(), TeamId::TeamName(n.to_string())))
                .collect(),
        }
    }

    fn num_remaining_questions(mut self, num_questions: usize) -> Self {
        self.questions.clear();

        for i in 1..=num_questions {
            let question = RawQuestion {
                url: format!("example url {}", i).to_owned(),
                answer: format!("example answer {}", i).to_owned(),
                acceptable_answers: Some("acceptable answer 1|acceptable answer 2".to_string()),
                category: format!("example category {}", i).to_owned(),
                score_value: ((i * 100) as u32),
                challenge: false,
                duration_seconds: None,
            }
            .into();

            self.questions.insert(question);
        }

        self
    }

    fn remaining_questions(mut self, questions: Vec<RawQuestion>) -> Self {
        self.questions = questions.into_iter().map(|q| q.into()).collect();
        self
    }

    fn voting_team(mut self, voting_team: Option<TeamId>) -> Self {
        self.voting_team = voting_team;
        self
    }

    fn max_vote_options(mut self, max_vote_options: usize) -> Self {
        self.max_vote_options = max_vote_options;
        self
    }

    fn build(self) -> Context {
        let duration = Duration::from_secs(10);

        let teams: TeamsHandle = Arc::new(RwLock::new(
            self.team_ids
                .iter()
                .enumerate()
                .map(|(team_index, (_n, team_id))| {
                    let mut team = Team::new(team_id.clone());
                    for i in 1..=2 {
                        let user_id = (team_index * 100 + i) as u64;
                        team.players.insert(UserId(user_id));
                    }
                    team
                })
                .collect(),
        ));
        let output = MockGameOutput::new(teams.clone());

        let state = VoteState::new(
            duration,
            &self.questions,
            self.voting_team.clone(),
            teams.clone(),
            output.clone(),
            self.max_vote_options,
        );

        Context {
            state,
            remaining_questions: self.questions,
            output,
            teams,
        }
    }
}

#[test]
fn display_last_remaining_questions() {
    let max_vote_options = 4;
    let remaining_questions_count = 3;

    let ctx = ContextBuilder::new()
        .max_vote_options(max_vote_options)
        .num_remaining_questions(remaining_questions_count)
        .build();

    assert_eq!(ctx.remaining_questions.len(), remaining_questions_count);
    assert_eq!(ctx.state.vote_options.len(), remaining_questions_count);
}

#[test]
fn display_no_more_than_max_questions() {
    let max_vote_options = 2;
    let remaining_questions_count = 3;

    let ctx = ContextBuilder::new()
        .max_vote_options(max_vote_options)
        .num_remaining_questions(remaining_questions_count)
        .build();

    assert_eq!(ctx.state.vote_options.len(), max_vote_options);
}

#[test]
fn displays_question_values() {
    let mut ctx = ContextBuilder::new()
        .remaining_questions(vec![
            RawQuestion {
                url: "example url 1".to_owned(),
                answer: "example answer 1".to_owned(),
                acceptable_answers: Some("acceptable answer 1|acceptable answer 2".to_string()),
                category: "cat A".to_owned(),
                score_value: 100,
                challenge: false,
                duration_seconds: None,
            },
            RawQuestion {
                url: "example url 2".to_owned(),
                answer: "example answer 2".to_owned(),
                acceptable_answers: Some("acceptable answer 1|acceptable answer 2".to_string()),
                category: "cat B".to_owned(),
                score_value: 200,
                challenge: true,
                duration_seconds: None,
            },
        ])
        .build();

    ctx.state.on_begin();

    let messages = ctx.state.output.flush();
    let message = messages
        .into_iter()
        .find_map(|message| match message {
            Entry::Text(TextEntry {
                message: Message::VotePoll(v),
                message_id: _,
            }) => Some(v),
            _ => None,
        })
        .unwrap();

    assert!(message
        .iter()
        .any(|(_, category, score)| category == "cat A" && *score == 100));

    assert!(message
        .iter()
        .any(|(_, category, score)| category == "cat B" && *score == 200));
}

#[test]
fn only_one_team_can_vote() {
    let builder = ContextBuilder::new();
    let red = builder.team_ids.get("red").unwrap();
    let blue = builder.team_ids.get("blue").unwrap();

    let mut ctx = ContextBuilder::new().voting_team(Some(red.clone())).build();

    let red_user_id = ctx
        .teams
        .read()
        .iter()
        .find(|team| (*team).id == *red)
        .unwrap();

    let blue_user_id = ctx
        .teams
        .read()
        .iter()
        .find(|team| (*team).id == *blue)
        .unwrap();

    ctx.state.on_begin();

    message_id = ctx.output.find(message);

    // ctx.react_to_message(message_id, reaction, red_user_id);
    // ctx.react_to_message(message_id, reaction, blue_user_id);
}

// #[test]
// fn all_teams_can_vote() {
// let builder = ContextBuilder::new();
// let red = builder.team_ids.get("red").unwrap();
// let blue = builder.team_ids.get("blue").unwrap();
//     let mut ctx = ContextBuilder::new().voting_team(None).build();
// }
