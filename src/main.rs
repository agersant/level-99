use anyhow::Result;
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

struct QuizzManager;
impl TypeMapKey for QuizzManager {
    type Value = Arc<Mutex<Quizz>>;
}

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

mod commands;
mod quizz;

use quizz::*;

fn main() -> Result<()> {
    let token = env::var("DISCORD_TOKEN_LEVEL99").expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler).expect("Err creating client");

    let quizz_source = std::path::Path::new("ExampleQuizz.csv");
    let quizz_definition = QuizzDefinition::open(quizz_source)?;
    let quizz = Quizz::new(quizz_definition);

    {
        let mut data = client.data.write();
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<QuizzManager>(Arc::new(Mutex::new(quizz)));
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&commands::GENERAL_GROUP),
    );

    let _ = client
        .start()
        .map_err(|why| println!("Client ended: {:?}", why));

    Ok(())
}
