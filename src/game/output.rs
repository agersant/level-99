use anyhow::*;
use serenity::http::client::Http;
use serenity::model::id::ChannelId;
use serenity::prelude::Mutex;
use std::sync::Arc;

#[derive(Clone, Hash)]
pub enum Payload {
    Text(String),
}

#[derive(Clone)]
pub struct OutputEvent {
    payload: Payload,
    channel: ChannelId,
}

pub struct DiscordOutput {
    http: Arc<Http>,
}

impl DiscordOutput {
    pub fn new(http: &Arc<Http>) -> Self {
        DiscordOutput {
            http: Arc::clone(http),
        }
    }

    pub fn broadcast(&self, event: OutputEvent) -> Result<()> {
        match event.payload {
            Payload::Text(s) => {
                event.channel.say(&self.http, s)?;
                Ok(())
            }
        }
    }
}

pub struct OutputPipe {
    channel: ChannelId,
    discord_output: Arc<Mutex<DiscordOutput>>,
}

impl OutputPipe {
    pub fn new(channel: ChannelId, discord_output: &Arc<Mutex<DiscordOutput>>) -> OutputPipe {
        OutputPipe {
            channel,
            discord_output: Arc::clone(discord_output),
        }
    }

    pub fn push(&mut self, payload: Payload) {
        let event = OutputEvent {
            payload,
            channel: self.channel,
        };
        let discord_output = self.discord_output.lock();
        if let Err(e) = discord_output.broadcast(event) {
            eprintln!("Broadcast error: {}", e);
        }
    }
}
