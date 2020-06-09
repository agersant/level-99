use anyhow::Result;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::client::Context;
use serenity::prelude::{Mutex, TypeMapKey};
use serenity::{
    client::{Client, EventHandler},
    framework::StandardFramework,
    model::gateway::Ready,
};
use std::env;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod commands;
mod game;

use crate::game::output::DiscordOutput;
use crate::game::pool::Pool as GamePool;

struct VoiceManager;
impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct DiscordOutputManager;
impl TypeMapKey for DiscordOutputManager {
    type Value = Arc<Mutex<DiscordOutput>>;
}

impl TypeMapKey for GamePool {
    type Value = Arc<GamePool>;
}

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() -> Result<()> {
    // Create game pool
    let game_pool = Arc::new(GamePool::default());
    let game_pool_for_ticker = Arc::clone(&game_pool);
    let _game_ticker = thread::spawn(move || {
        let manager = game_pool_for_ticker.clone();
        let mut last_tick_time = Instant::now();
        loop {
            let now = Instant::now();
            let dt = now.duration_since(last_tick_time);
            last_tick_time = now;
            manager.tick(dt);
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    // Create discord client
    let token = env::var("DISCORD_TOKEN_LEVEL99").expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    // Create output
    let discord_output = DiscordOutput::new(&client.cache_and_http.http);
    let discord_output: Arc<Mutex<DiscordOutput>> = Arc::new(Mutex::new(discord_output));

    // Associate persistent data with discord client
    {
        let mut data = client.data.write();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<DiscordOutputManager>(Arc::clone(&discord_output));
        data.insert::<GamePool>(Arc::clone(&game_pool));
    }

    // Configure discord client
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&commands::GENERAL_GROUP),
    );

    // Run discord client
    if let Err(why) = client.start() {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
