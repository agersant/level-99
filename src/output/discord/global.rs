use anyhow::*;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::http::client::Http;
use serenity::model::channel::ReactionType;
use serenity::model::id::GuildId;
use serenity::model::id::{ChannelId, MessageId, UserId};
use serenity::prelude::Mutex;
use serenity::voice;
use serenity::voice::LockedAudio;
use std::path::Path;
use std::sync::Arc;

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

    pub fn say(&self, channel_id: ChannelId, content: &str) -> Result<MessageId> {
        let message = channel_id.say(&self.http, content)?;
        Ok(message.id)
    }

    pub fn say_with_reactions(
        &self,
        channel_id: ChannelId,
        content: &str,
        reactions: &Vec<String>,
    ) -> Result<MessageId> {
        let reactions: Vec<ReactionType> = reactions
            .into_iter()
            .map(|r| ReactionType::Unicode(r.clone()))
            .collect();
        let message = channel_id.send_message(&self.http, |m| {
            m.content(content);
            m.reactions(reactions);
            m
        })?;
        Ok(message.id)
    }

    pub fn play_youtube_audio(&self, guild_id: GuildId, url: String) -> Result<LockedAudio> {
        let mut manager = self.client_voice_manager.lock();
        if let Some(handler) = manager.get_mut(guild_id) {
            let source = voice::ytdl(&url)?;
            Ok(handler.play_returning(source))
        } else {
            Err(anyhow!("Not in a voice channel to play in"))
        }
    }

    pub fn play_file_audio(&self, guild_id: GuildId, path: &Path) -> Result<LockedAudio> {
        let mut manager = self.client_voice_manager.lock();
        if let Some(handler) = manager.get_mut(guild_id) {
            let source = voice::ffmpeg(path)?;
            Ok(handler.play_returning(source))
        } else {
            Err(anyhow!("Not in a voice channel to play in"))
        }
    }

    pub fn stop_audio(&self, guild_id: GuildId) -> Result<()> {
        let mut manager = self.client_voice_manager.lock();
        if let Some(handler) = manager.get_mut(guild_id) {
            handler.stop();
            Ok(())
        } else {
            Err(anyhow!("Not in a voice channel to play in"))
        }
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
