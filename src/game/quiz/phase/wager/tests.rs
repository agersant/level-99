use super::*;

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

use crate::game::quiz::definition::RawQuestion;
use crate::game::team::Team;
use crate::output::mock::MockGameOutput;

struct Context {
    state: WagerState<MockGameOutput>,
    // output: MockGameOutput,
}

struct ContextBuilder {
    participants: HashSet<TeamId>,
    team_ids: HashMap<String, TeamId>,
}

impl ContextBuilder {
    fn new() -> Self {
        ContextBuilder {
            participants: HashSet::new(),
            team_ids: ["red", "blue"]
                .iter()
                .map(|n| (n.to_string(), TeamId::TeamName(n.to_string())))
                .collect(),
        }
    }

    fn participants(mut self, participants: HashSet<TeamId>) -> Self {
        self.participants = participants;
        self
    }

    fn all_teams_participate(mut self) -> Self {
        self.participants = self.team_ids.values().cloned().collect();
        self
    }

    fn build(self) -> Context {
        let question = RawQuestion {
            url: "example url".to_owned(),
            answer: "example answer".to_owned(),
            acceptable_answers: Some("acceptable answer 1|acceptable answer 2".to_string()),
            category: "example category".to_owned(),
            score_value: 100,
            challenge: true,
            duration_seconds: None,
        }
        .into();
        let duration = Duration::from_secs(10);
        let teams: TeamsHandle = Arc::new(RwLock::new(
            self.team_ids
                .iter()
                .map(|(_n, team_id)| Team::new(team_id.clone()))
                .collect(),
        ));
        let output = MockGameOutput::new(teams.clone());
        let max_question_score_value = 2000;

        let state = WagerState::new(
            question,
            duration,
            teams,
            output,
            self.participants,
            max_question_score_value,
        );

        Context { state }
    }
}

#[test]
fn one_team_one_wager() {
    let builder = ContextBuilder::new();
    let mut participants = HashSet::new();
    let red = builder.team_ids.get("red").unwrap();
    participants.insert(red.clone());
    let mut ctx = ContextBuilder::new().participants(participants).build();
    let wager = 1000;

    assert!(ctx.state.wager(&red, wager).is_ok());

    assert!(ctx.state.is_over());
    assert_eq!(*ctx.state.wager_amounts.get(&red).unwrap(), wager);
}

#[test]
fn no_wager() {
    let mut ctx = ContextBuilder::new().all_teams_participate().build();
    let builder = ContextBuilder::new();
    let red = builder.team_ids.get("red").unwrap();

    assert!(!ctx.state.is_over());
    ctx.state.on_begin();
    assert!(!ctx.state.is_over());
    ctx.state.on_tick(Duration::from_secs(30));
    assert!(ctx.state.is_over());
    ctx.state.on_end();
    assert_eq!(
        ctx.state.wager_amounts.get(&red).unwrap(),
        &ctx.state.question.score_value
    );
}

#[test]
fn one_team_wager_too_big() {
    let builder = ContextBuilder::new();
    let mut participants = HashSet::new();
    let red = builder.team_ids.get("red").unwrap();
    participants.insert(red.clone());
    let mut ctx = ContextBuilder::new().participants(participants).build();

    assert!(ctx.state.wager(&red, 10_000).is_ok());
    assert!(ctx.state.is_over());
    assert_eq!(
        *ctx.state.wager_amounts.get(&red).unwrap(),
        2 * ctx.state.max_question_score_value
    );
}

#[test]
fn one_team_wager_too_small() {
    let builder = ContextBuilder::new();
    let mut participants = HashSet::new();
    let red = builder.team_ids.get("red").unwrap();
    participants.insert(red.clone());
    let mut ctx = ContextBuilder::new().participants(participants).build();

    assert!(ctx.state.wager(&red, 10).is_ok());

    assert!(ctx.state.is_over());
    assert_eq!(
        *ctx.state.wager_amounts.get(&red).unwrap(),
        ctx.state.question.score_value
    );
}

#[test]
fn two_team_with_wagers() {
    let builder = ContextBuilder::new();
    let red = builder.team_ids.get("red").unwrap();
    let blue = builder.team_ids.get("blue").unwrap();
    let mut ctx = ContextBuilder::new().all_teams_participate().build();
    let red_wager = 1000;
    let blue_wager = 2000;

    assert!(ctx.state.wager(&red, red_wager).is_ok());
    assert!(!ctx.state.is_over());
    assert!(ctx.state.wager(&blue, blue_wager).is_ok());
    assert!(ctx.state.is_over());

    assert_eq!(ctx.state.wager_amounts.get(&red).unwrap(), &red_wager);
    assert_eq!(ctx.state.wager_amounts.get(&blue).unwrap(), &blue_wager);
}
