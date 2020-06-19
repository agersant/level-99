use parking_lot::RwLock;
use serenity::client::Context as SerenityContext;
use serenity::model::id::GuildId;
use serenity::prelude::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::game::Game;
use crate::output::{OutputHandle, OutputPipe};
use crate::DiscordOutputManager;

#[derive(Default)]
pub struct Pool {
    games: RwLock<HashMap<GuildId, Arc<Mutex<Game>>>>,
}

impl Pool {
    pub fn get_game(&self, ctx: &SerenityContext, guild_id: GuildId) -> Arc<Mutex<Game>> {
        let game_exists = {
            let map = self.games.read();
            map.contains_key(&guild_id)
        };
        if !game_exists {
            let discord_output = ctx
                .data
                .read()
                .get::<DiscordOutputManager>()
                .cloned()
                .expect("Expected DiscordOutput in ShareMap.");

            let teams = Arc::new(RwLock::new(Vec::new()));
            let output = OutputHandle::new(OutputPipe::new(guild_id, &discord_output));
            let game = Game::new(output, teams);
            let mut map = self.games.write();
            map.insert(guild_id, Arc::new(Mutex::new(game)));
        }
        let map = self.games.read();
        Arc::clone(map.get(&guild_id).unwrap())
    }

    pub fn tick(&self, dt: Duration) {
        let map = self.games.read();
        for (_channel, game) in map.iter() {
            let mut game = game.lock();
            game.tick(dt);
        }
    }
}
