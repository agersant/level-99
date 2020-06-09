use anyhow::*;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::http::client::Http;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use serenity::prelude::Mutex;
use serenity::voice;
use std::sync::Arc;

#[derive(Clone, Hash)]
pub enum Payload {
    Text(String),
    Audio(String),
    StopAudio,
}

#[derive(Clone)]
pub struct OutputEvent {
    payload: Payload,
    channel: ChannelId,
    guild: GuildId,
}

pub struct DiscordOutput {
    http: Arc<Http>,
    client_voice_manager: Arc<Mutex<ClientVoiceManager>>,
}

impl DiscordOutput {
    pub fn new(http: Arc<Http>, client_voice_manager: Arc<Mutex<ClientVoiceManager>>) -> Self {
        DiscordOutput {
            http,
            client_voice_manager,
        }
    }

    pub fn broadcast(&self, event: OutputEvent) -> Result<()> {
        match event.payload {
            Payload::Text(s) => {
                event.channel.say(&self.http, s)?;
                Ok(())
            }
            Payload::Audio(url) => {
                let mut manager = self.client_voice_manager.lock();
                if let Some(handler) = manager.get_mut(event.guild) {
                    let source = voice::ytdl(&url)?;
                    handler.play_only(source);
                } else {
                    eprintln!("Not in a voice channel to play in");
                }
                Ok(())
            }
            Payload::StopAudio => {
                let mut manager = self.client_voice_manager.lock();
                if let Some(handler) = manager.get_mut(event.guild) {
                    handler.stop();
                } else {
                    eprintln!("Not in a voice channel to play in");
                }
                Ok(())
            }
        }
    }
}

pub struct OutputPipe {
    guild: GuildId,
    channel: ChannelId,
    discord_output: Arc<Mutex<DiscordOutput>>,
}

impl OutputPipe {
    pub fn new(
        guild: GuildId,
        channel: ChannelId,
        discord_output: &Arc<Mutex<DiscordOutput>>,
    ) -> OutputPipe {
        OutputPipe {
            guild,
            channel,
            discord_output: Arc::clone(discord_output),
        }
    }

    pub fn push(&mut self, payload: Payload) {
        let event = OutputEvent {
            payload,
            guild: self.guild,
            channel: self.channel,
        };
        let discord_output = self.discord_output.lock();
        if let Err(e) = discord_output.broadcast(event) {
            eprintln!("Broadcast error: {}", e);
        }
    }
}
