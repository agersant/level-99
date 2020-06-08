use serenity::model::id::GuildId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use crate::game::Game;

#[derive(Default)]
pub struct Manager {
    games: RwLock<HashMap<GuildId, Arc<Mutex<Game>>>>,
}

impl Manager {
    pub fn get_game(&self, guild: GuildId) -> Arc<Mutex<Game>> {
        let game_exists = {
            let map = self.games.read().expect("Manager RwLock poisoned");
            map.contains_key(&guild)
        };
        if !game_exists {
            let mut map = self.games.write().expect("Manager RwLock poisoned");
            map.insert(guild, Arc::new(Mutex::new(Game::new())));
        }
        let map = self.games.read().expect("Manager RwLock poisoned");
        Arc::clone(map.get(&guild).expect("Manager missing game"))
    }
}
