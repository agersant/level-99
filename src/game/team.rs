use parking_lot::RwLock;
use serenity::model::id::UserId;
use std::collections::HashSet;
use std::sync::Arc;

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
