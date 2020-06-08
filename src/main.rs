use std::{env, sync::Arc};

use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::{client::Context, prelude::Mutex};

use serenity::{
    client::{Client, EventHandler},
    framework::StandardFramework,
    model::gateway::Ready,
};

use serenity::prelude::*;

struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

mod commands;
mod quizz;

fn main() {
    let token = env::var("DISCORD_TOKEN_LEVEL99").expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    {
        let mut data = client.data.write();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&commands::GENERAL_GROUP),
    );

    let _ = client
        .start()
        .map_err(|why| println!("Client ended: {:?}", why));
}
