use anyhow::*;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use regex::Regex;
use serenity::model::id::UserId;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;
use unidecode::unidecode;

lazy_static! {
    static ref FORBIDDEN_TEAM_NAME_CHARACTERS_REGEX: Regex = Regex::new("[^\\sa-z0-9-]").unwrap();
    static ref WHITESPACE_REGEX: Regex = Regex::new("\\s+").unwrap();
}

pub fn sanitize_name(name: &str) -> Result<String> {
    let name = unidecode(name);

    let name: String = FORBIDDEN_TEAM_NAME_CHARACTERS_REGEX
        .replace_all(&name.to_lowercase(), "")
        .into();

    let name = name.trim();
    if name.is_empty() {
        return Err(anyhow!("Invalid team name"));
    }

    let name: String = WHITESPACE_REGEX.replace_all(&name, "-").into();
    Ok(name)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TeamId {
    TeamName(String),
}

#[derive(Clone, Debug)]
pub struct Team {
    pub id: TeamId,
    pub players: HashSet<UserId>,
    pub score: i32,
}

impl Team {
    pub fn new(id: TeamId) -> Self {
        Team {
            id,
            score: 0,
            players: HashSet::new(),
        }
    }

    pub fn get_display_name(&self) -> &str {
        match &self.id {
            TeamId::TeamName(name) => &name,
        }
    }

    pub fn update_score(&mut self, delta: i32) {
        self.score += delta;
    }
}

pub type TeamsHandle = Arc<RwLock<Vec<Team>>>;
