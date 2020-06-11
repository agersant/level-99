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

mod channels;
mod commands;
mod game;
mod output;

use crate::game::pool::Pool as GamePool;
use crate::output::DiscordOutput;

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
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        for guild in &ready.guilds {
            let guild_id = guild.id();
            let game_pool = ctx
                .data
                .read()
                .get::<GamePool>()
                .cloned()
                .expect("Expected GamePool in ShareMap.");
            let game_lock = game_pool.get_game(&ctx, guild_id);
            let game = game_lock.lock();
            if let Err(e) = channels::update_team_channels(&ctx, guild_id, game.get_teams()) {
                eprintln!("Could not initialize team channels: {:#}", e);
            }
        }
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
    let discord_output = DiscordOutput::new(
        Arc::clone(&client.cache_and_http.http),
        Arc::clone(&client.voice_manager),
    );
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
            .group(&commands::quizzmaster::MAIN_GROUP)
            .group(&commands::quizzmaster::RESET_GROUP)
            .group(&commands::player::MAIN_GROUP),
    );

    // Run discord client
    if let Err(why) = client.start() {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
