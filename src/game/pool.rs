use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::prelude::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::game::output::OutputPipe;
use crate::game::Game;
use crate::DiscordOutputManager;

#[derive(Default)]
pub struct Pool {
    games: RwLock<HashMap<ChannelId, Arc<Mutex<Game>>>>,
}

impl Pool {
    pub fn get_game(&self, ctx: &Context, channel: ChannelId) -> Arc<Mutex<Game>> {
        let game_exists = {
            let map = self.games.read().expect("Pool RwLock poisoned");
            map.contains_key(&channel)
        };
        if !game_exists {
            let discord_output = ctx
                .data
                .read()
                .get::<DiscordOutputManager>()
                .cloned()
                .expect("Expected DiscordOutput in ShareMap.");
            let dispatcher = OutputPipe::new(channel, &discord_output);
            let mut map = self.games.write().expect("Pool RwLock poisoned");
            map.insert(channel, Arc::new(Mutex::new(Game::new(dispatcher))));
        }
        let map = self.games.read().expect("Pool RwLock poisoned");
        Arc::clone(map.get(&channel).expect("Pool missing game"))
    }

    pub fn tick(&self, dt: Duration) {
        let map = self.games.read().expect("Pool RwLock poisoned");
        for (_channel, game) in map.iter() {
            let mut game = game.lock();
            game.tick(dt);
        }
    }
}
