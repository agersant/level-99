use anyhow::Result;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::prelude::*;
use serenity::{client::Context, prelude::Mutex};
use serenity::{
    client::{Client, EventHandler},
    framework::StandardFramework,
    model::gateway::Ready,
};
use std::{env, sync::Arc};

mod commands;
mod game;

use crate::game::manager::Manager;

struct VoiceManager;
impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

impl TypeMapKey for Manager {
    type Value = Arc<Manager>;
}

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() -> Result<()> {
    let token = env::var("DISCORD_TOKEN_LEVEL99").expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    let manager = Arc::new(Manager::default());

    {
        let mut data = client.data.write();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<Manager>(Arc::clone(&manager));
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&commands::GENERAL_GROUP),
    );

    if let Err(why) = client.start() {
        eprintln!("Client error: {:?}", why);
    }

    Ok(())
}
