use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

use super::*;
use crate::game::quiz::definition::{Question, RawQuestion};
use crate::game::team::Team;
use crate::output::mock::MockGameOutput;

struct ContextBuilder {
    question: RawQuestion,
    team_ids: HashMap<String, TeamId>,
    wagers: Option<HashMap<TeamId, u32>>,
}

impl ContextBuilder {
    fn new() -> Self {
        ContextBuilder {
            question: RawQuestion {
                url: "example url".to_owned(),
                answer: "example answer".to_owned(),
                acceptable_answers: None,
                category: "example category".to_owned(),
                score_value: 100,
                daily_double: false,
                duration_seconds: None,
            },
            team_ids: ["red", "green", "blue"]
                .iter()
                .map(|n| (n.to_string(), TeamId::TeamName(n.to_string())))
                .collect(),
            wagers: None,
        }
    }

    fn question(mut self, question: RawQuestion) -> Self {
        self.question = question;
        self
    }

    fn wager(mut self, team_id: &TeamId, amount: u32) -> Self {
        if self.wagers.is_none() {
            self.wagers = Some(HashMap::new());
        }
        self.wagers
            .as_mut()
            .unwrap()
            .insert(team_id.clone(), amount);
        self
    }

    fn build(self) -> Context {
        let output = MockGameOutput::new();
        let teams: TeamsHandle = Arc::new(RwLock::new(
            self.team_ids
                .iter()
                .map(|(_n, team_id)| Team::new(team_id.clone()))
                .collect(),
        ));

        let participants = teams.read().iter().map(|t| t.id.clone()).collect();
        let question: Question = self.question.into();
        let duration = Duration::from_secs(10);
        let state = QuestionState::new(
            question.clone(),
            duration,
            teams.clone(),
            output.clone(),
            participants,
            self.wagers,
        );

        Context {
            state,
            team_ids: self.team_ids,
            teams,
            output,
        }
    }
}

struct Context {
    state: QuestionState<MockGameOutput>,
    team_ids: HashMap<String, TeamId>,
    teams: TeamsHandle,
    output: MockGameOutput,
}

impl Context {
    pub fn get_team_score(&self, team_id: &TeamId) -> i32 {
        self.teams
            .read()
            .iter()
            .find(|t| t.id == *team_id)
            .unwrap()
            .score
    }
}

#[test]
fn announces_question() {
    let mut ctx = ContextBuilder::new().build();
    ctx.state.on_begin();
    assert_eq!(
        ctx.output.flush(),
        [Message::QuestionBegins(ctx.state.question.clone())]
    );
}

#[test]
fn timeout_announces_answer() {
    let mut ctx = ContextBuilder::new().build();
    ctx.state.on_end();
    assert!(ctx
        .output
        .flush()
        .contains(&Message::TimeUp(ctx.state.question.clone())));
}

#[test]
fn timeout_announces_scores() {
    let mut ctx = ContextBuilder::new().build();
    let red = ctx.team_ids.get("red").unwrap();
    let green = ctx.team_ids.get("green").unwrap();
    let blue = ctx.team_ids.get("blue").unwrap();

    let red_score = 200;
    let green_score = 100;
    ctx.teams
        .write()
        .iter_mut()
        .find(|t| t.id == *red)
        .unwrap()
        .update_score(red_score);
    ctx.teams
        .write()
        .iter_mut()
        .find(|t| t.id == *green)
        .unwrap()
        .update_score(green_score);
    ctx.state.on_end();

    let expected_scores = vec![
        (red.clone(), red_score),
        (green.clone(), green_score),
        (blue.clone(), 0),
    ];
    assert!(ctx
        .output
        .flush()
        .contains(&Message::ScoresRecap(expected_scores)));
}

#[test]
fn times_out_after_duration() {
    let mut ctx = ContextBuilder::new().build();
    assert!(!ctx.state.is_over());
    ctx.state.on_begin();
    assert!(!ctx.state.is_over());
    ctx.state.on_tick(Duration::from_secs(5));
    assert!(!ctx.state.is_over());
    ctx.state.on_tick(Duration::from_secs(5));
    assert!(ctx.state.is_over());
}

#[test]
fn question_can_override_duration() {
    let builder = ContextBuilder::new();
    let mut question = builder.question.clone();
    question.duration_seconds = Some(100);
    let mut ctx = ContextBuilder::new().question(question).build();
    assert!(!ctx.state.is_over());
    ctx.state.on_begin();
    assert!(!ctx.state.is_over());
    ctx.state.on_tick(Duration::from_secs(99));
    assert!(!ctx.state.is_over());
    ctx.state.on_tick(Duration::from_secs(2));
    assert!(ctx.state.is_over());
}

#[test]
fn can_only_answer_once() {
    let mut ctx = ContextBuilder::new().build();
    let red = ctx.team_ids.get("red").unwrap();
    let blue = ctx.team_ids.get("blue").unwrap();
    assert!(ctx.state.guess(&blue, "random guess").is_ok());
    assert!(ctx.state.guess(&red, "random guess").is_ok());
    assert!(ctx.state.guess(&blue, "random guess").is_err());
    assert!(ctx.state.guess(&red, "random guess").is_err());
}

#[test]
fn only_participants_can_answer() {
    let mut ctx = ContextBuilder::new().build();
    let yellow = TeamId::TeamName("yellow".into());
    assert!(ctx.state.guess(&yellow, "anything").is_err());
}

#[test]
fn wrong_answer_deducts_points() {
    let mut ctx = ContextBuilder::new().build();
    let red = ctx.team_ids.get("red").unwrap().clone();
    assert!(ctx
        .state
        .guess(&red, &ctx.state.question.answer[1..].to_string())
        .is_ok());
    let score = ctx.teams.read().iter().find(|t| t.id == red).unwrap().score;
    assert!(score < 0);
    assert_eq!(-1 * ctx.state.question.score_value as i32, score);
}

#[test]
fn correct_answer_gives_points() {
    let mut ctx = ContextBuilder::new().build();
    let red = ctx.team_ids.get("red").unwrap().clone();
    assert!(ctx
        .state
        .guess(&red, &ctx.state.question.answer.to_string())
        .is_ok());
    let score = ctx.teams.read().iter().find(|t| t.id == red).unwrap().score;
    assert!(score > 0);
    assert_eq!(ctx.state.question.score_value as i32, score);
}

#[test]
fn only_first_answer_gets_full_points() {
    let mut ctx = ContextBuilder::new().build();
    let red = ctx.team_ids.get("red").unwrap().clone();
    let green = ctx.team_ids.get("green").unwrap().clone();
    let blue = ctx.team_ids.get("blue").unwrap().clone();

    assert!(ctx
        .state
        .guess(&red, &ctx.state.question.answer[1..].to_string())
        .is_ok());
    assert_eq!(
        -1 * ctx.state.question.score_value as i32,
        ctx.get_team_score(&red)
    );

    assert!(ctx
        .state
        .guess(&blue, &ctx.state.question.answer.to_string())
        .is_ok());
    assert_eq!(
        ctx.state.question.score_value as i32 / 2,
        ctx.get_team_score(&blue)
    );

    assert!(ctx
        .state
        .guess(&green, &ctx.state.question.answer[1..].to_string())
        .is_ok());
    assert_eq!(
        ctx.state.question.score_value as i32 / -2,
        ctx.get_team_score(&green)
    );
}

#[test]
fn wager_defaults_to_question_value() {
    let builder = ContextBuilder::new();
    let mut question = builder.question.clone();
    question.daily_double = true;
    let mut ctx = ContextBuilder::new().question(question).build();
    let red = ctx.team_ids.get("red").unwrap().clone();

    assert!(ctx
        .state
        .guess(&red, &ctx.state.question.answer.to_string())
        .is_ok());
    let score = ctx.teams.read().iter().find(|t| t.id == red).unwrap().score;
    assert_eq!(ctx.state.question.score_value as i32, score);
}

#[test]
fn wager_overrides_question_value() {
    let builder = ContextBuilder::new();
    let mut question = builder.question.clone();
    question.daily_double = true;
    let red = builder.team_ids.get("red").unwrap().clone();
    let blue = builder.team_ids.get("blue").unwrap().clone();
    let red_wager_amount = 1000;
    let blue_wager_amount = 2000;
    let mut ctx = ContextBuilder::new()
        .question(question)
        .wager(&red, red_wager_amount)
        .wager(&blue, blue_wager_amount)
        .build();

    let score = ctx.teams.read().iter().find(|t| t.id == red).unwrap().score;
    assert_eq!(0, score);

    assert!(ctx
        .state
        .guess(&red, &ctx.state.question.answer.to_string())
        .is_ok());
    let score = ctx.teams.read().iter().find(|t| t.id == red).unwrap().score;
    assert_eq!(red_wager_amount as i32, score);

    assert!(ctx
        .state
        .guess(&blue, &ctx.state.question.answer[1..].to_string())
        .is_ok());
    let score = ctx
        .teams
        .read()
        .iter()
        .find(|t| t.id == blue)
        .unwrap()
        .score;
    assert_eq!(blue_wager_amount as i32 * -1, score);
}

#[test]
fn reveals_answer_after_all_teams_have_guessed() {
    let mut ctx = ContextBuilder::new().build();
    let red = ctx.team_ids.get("red").unwrap().clone();
    let green = ctx.team_ids.get("green").unwrap().clone();
    let blue = ctx.team_ids.get("blue").unwrap().clone();

    let is_answer_reveal = |m: &Message| match m {
        Message::AnswerReveal(_) => true,
        _ => false,
    };
    assert!(ctx.state.guess(&red, "whatever").is_ok());
    assert!(ctx.state.guess(&green, "whatever").is_ok());
    assert!(!ctx.output.flush().iter().any(is_answer_reveal));
    assert!(ctx.state.guess(&blue, "whatever").is_ok());
    assert!(ctx.output.flush().iter().any(is_answer_reveal));
}
