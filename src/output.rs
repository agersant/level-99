use anyhow::*;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::http::client::Http;
use serenity::model::channel::ReactionType;
use serenity::model::id::{ChannelId, GuildId, MessageId, UserId};
use serenity::prelude::Mutex;
use serenity::voice;
use std::sync::Arc;

#[derive(Clone, Hash)]
pub enum Payload {
    Text(String),
    TextWithReactions(String, Vec<String>),
    Audio(String),
    StopAudio,
}

#[derive(Clone)]
pub struct OutputEvent {
    payload: Payload,
    channel_id: ChannelId,
    guild_id: GuildId,
}

pub enum OutputResult {
    Message(MessageId),
}

pub struct DiscordOutput {
    http: Arc<Http>,
    client_voice_manager: Arc<Mutex<ClientVoiceManager>>,
}

impl std::fmt::Debug for DiscordOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscordOutput").finish()
    }
}

impl DiscordOutput {
    pub fn new(http: Arc<Http>, client_voice_manager: Arc<Mutex<ClientVoiceManager>>) -> Self {
        DiscordOutput {
            http,
            client_voice_manager,
        }
    }

    pub fn broadcast(&self, event: OutputEvent) -> Result<Option<OutputResult>> {
        let result = match event.payload {
            Payload::Text(content) => {
                let message = event.channel_id.say(&self.http, content)?;
                Some(OutputResult::Message(message.id))
            }
            Payload::TextWithReactions(content, reactions) => {
                let reactions: Vec<ReactionType> = reactions
                    .into_iter()
                    .map(|r| ReactionType::Unicode(r))
                    .collect();
                let message = event.channel_id.send_message(&self.http, |m| {
                    m.content(content);
                    m.reactions(reactions);
                    m
                })?;
                Some(OutputResult::Message(message.id))
            }
            Payload::Audio(url) => {
                let mut manager = self.client_voice_manager.lock();
                if let Some(handler) = manager.get_mut(event.guild_id) {
                    let source = voice::ytdl(&url)?;
                    handler.play_only(source);
                } else {
                    eprintln!("Not in a voice channel to play in");
                }
                None
            }
            Payload::StopAudio => {
                let mut manager = self.client_voice_manager.lock();
                if let Some(handler) = manager.get_mut(event.guild_id) {
                    handler.stop();
                } else {
                    eprintln!("Not in a voice channel to play in");
                }
                None
            }
        };
        Ok(result)
    }

    pub fn read_reactions(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<Vec<UserId>> {
        channel_id
            .reaction_users(
                &self.http,
                message_id,
                ReactionType::Unicode(reaction),
                None,
                None,
            )
            .map(|v| v.iter().map(|u| u.id).collect())
            .map_err(|e| Error::new(e))
    }
}

#[derive(Debug)]
pub struct OutputPipe {
    guild_id: GuildId,
    channel_id: ChannelId,
    discord_output: Arc<Mutex<DiscordOutput>>,
}

impl OutputPipe {
    pub fn new(
        guild_id: GuildId,
        channel_id: ChannelId,
        discord_output: &Arc<Mutex<DiscordOutput>>,
    ) -> OutputPipe {
        OutputPipe {
            guild_id,
            channel_id,
            discord_output: Arc::clone(discord_output),
        }
    }

    pub fn push(&mut self, payload: Payload) -> Option<OutputResult> {
        let event = OutputEvent {
            payload,
            guild_id: self.guild_id,
            channel_id: self.channel_id,
        };
        let discord_output = self.discord_output.lock();
        match discord_output.broadcast(event) {
            Ok(output_result) => output_result,
            Err(e) => {
                eprintln!("Broadcast error: {}", e);
                None
            }
        }
    }

    pub fn read_reactions(&self, message_id: MessageId, reaction: String) -> Result<Vec<UserId>> {
        let discord_output = self.discord_output.lock();
        discord_output.read_reactions(self.channel_id, message_id, reaction)
    }
}
