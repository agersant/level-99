use anyhow::*;
use serenity::client::Context as SerenityContext;
use serenity::model::id::{ChannelId, GuildId};
use serenity::prelude::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::game::output::OutputPipe;
use crate::game::Game;
use crate::DiscordOutputManager;

#[derive(Default)]
pub struct Pool {
    games: RwLock<HashMap<GuildId, Arc<Mutex<Game>>>>,
}

impl Pool {
    pub fn get_game(&self, ctx: &SerenityContext, channel: ChannelId) -> Result<Arc<Mutex<Game>>> {
        let guild = ctx
            .cache
            .read()
            .guild_channel(channel)
            .context("Server not found")?
            .read()
            .guild_id;

        let game_exists = {
            let map = self.games.read().expect("Pool RwLock poisoned");
            map.contains_key(&guild)
        };
        if !game_exists {
            let discord_output = ctx
                .data
                .read()
                .get::<DiscordOutputManager>()
                .cloned()
                .expect("Expected DiscordOutput in ShareMap.");

            let dispatcher = OutputPipe::new(guild, channel, &discord_output);
            let mut map = self.games.write().expect("Pool RwLock poisoned");
            map.insert(guild, Arc::new(Mutex::new(Game::new(dispatcher))));
        }
        let map = self.games.read().expect("Pool RwLock poisoned");
        Ok(Arc::clone(map.get(&guild).expect("Pool missing game")))
    }

    pub fn tick(&self, dt: Duration) {
        let map = self.games.read().expect("Pool RwLock poisoned");
        for (_channel, game) in map.iter() {
            let mut game = game.lock();
            game.tick(dt);
        }
    }
}
