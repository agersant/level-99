use anyhow::*;
use parking_lot::RwLock;
use regex::Regex;
use serenity::model::id::{ChannelId, UserId};
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;
use unidecode::unidecode;

pub fn sanitize_name(name: &str) -> Result<String> {
    let name = unidecode(name);

    let forbidden_characters = Regex::new("[^\\sa-z0-9-]").unwrap(); // TODO avoid recompiling this regex everytime
    let name: String = forbidden_characters
        .replace_all(&name.to_lowercase(), "")
        .into();

    let name = name.trim();
    if name.is_empty() {
        return Err(anyhow!("Invalid team name"));
    }

    let whitespace = Regex::new("\\s+").unwrap(); // TODO avoid recompiling this regex everytime
    let name: String = whitespace.replace_all(&name, "-").into();
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
    pub channel_id: Option<ChannelId>,
    pub score: i32,
}

impl Team {
    pub fn new(id: TeamId) -> Self {
        Team {
            id,
            score: 0,
            players: HashSet::new(),
            channel_id: None,
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
