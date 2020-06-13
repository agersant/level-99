use anyhow::*;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::http::client::Http;
use serenity::model::channel::ReactionType;
use serenity::model::id::{ChannelId, GuildId, MessageId, UserId};
use serenity::prelude::Mutex;
use serenity::voice;
use serenity::voice::LockedAudio;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::game::team::TeamId;

pub enum Recipient {
    AllTeams,
    Team(TeamId),
    AllTeamsExcept(TeamId),
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

// TODO Keep more references to the Arc<RwLock<OutputPipe>> instead of passing it to a bunch of functions
#[derive(Debug)]
pub struct OutputPipe {
    guild_id: GuildId,
    discord_output: Arc<Mutex<DiscordOutput>>,
    team_channels: HashMap<TeamId, ChannelId>,
}

impl OutputPipe {
    pub fn new(guild_id: GuildId, discord_output: &Arc<Mutex<DiscordOutput>>) -> OutputPipe {
        OutputPipe {
            guild_id,
            discord_output: Arc::clone(discord_output),
            team_channels: HashMap::new(),
        }
    }

    pub fn update_team_channels(&mut self, channel_ids: HashMap<TeamId, ChannelId>) {
        self.team_channels = channel_ids;
    }

    fn get_team_channel(&self, team_id: &TeamId) -> Result<ChannelId> {
        match self.team_channels.get(team_id) {
            Some(channel_id) => Ok(channel_id.clone()),
            None => Err(anyhow!("Team has no channel")),
        }
    }

    pub fn say(
        &self,
        recipient: &Recipient,
        content: &str,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let mut message_ids = HashMap::new();
        match recipient {
            Recipient::Team(team_id) => {
                let discord_output = self.discord_output.lock();
                let channel_id = self.get_team_channel(team_id);
                let channel_id_copy = match &channel_id {
                    Ok(c) => Ok(c.clone()),
                    Err(_e) => Err(anyhow!("Team has no channel")),
                };
                let message_id = channel_id_copy.and_then(|c| discord_output.say(c, content));
                match (channel_id, message_id) {
                    (Ok(c), Ok(m)) => message_ids.insert(team_id.clone(), Ok((c, m))),
                    _ => message_ids
                        .insert(team_id.clone(), Err(anyhow!("Could not send team message"))),
                };
            }
            Recipient::AllTeams => {
                for (team_id, _channel_id) in self.team_channels.iter() {
                    message_ids.extend(self.say(&Recipient::Team(team_id.clone()), content));
                }
            }
            Recipient::AllTeamsExcept(team_id) => {
                for (team_id, _channel_id) in
                    self.team_channels.iter().filter(|(t, _c)| t != &team_id)
                {
                    message_ids.extend(self.say(&Recipient::Team(team_id.clone()), content));
                }
            }
        }
        message_ids
    }

    pub fn say_with_reactions(
        &self,
        recipient: &Recipient,
        content: &str,
        reactions: &Vec<String>,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let mut message_ids = HashMap::new();
        match recipient {
            Recipient::Team(team_id) => {
                let discord_output = self.discord_output.lock();
                let channel_id = self.get_team_channel(team_id);
                let channel_id_copy = match &channel_id {
                    Ok(c) => Ok(c.clone()),
                    Err(_e) => Err(anyhow!("Team has no channel")),
                };
                let message_id = channel_id_copy
                    .and_then(|c| discord_output.say_with_reactions(c, content, reactions));
                match (channel_id, message_id) {
                    (Ok(c), Ok(m)) => message_ids.insert(team_id.clone(), Ok((c, m))),
                    _ => message_ids
                        .insert(team_id.clone(), Err(anyhow!("Could not send team message"))),
                };
            }
            Recipient::AllTeams => {
                for (team_id, _channel_id) in self.team_channels.iter() {
                    message_ids.extend(self.say_with_reactions(
                        &Recipient::Team(team_id.clone()),
                        content,
                        reactions,
                    ));
                }
            }
            Recipient::AllTeamsExcept(team_id) => {
                for (team_id, _channel_id) in
                    self.team_channels.iter().filter(|(t, _c)| t != &team_id)
                {
                    message_ids.extend(self.say_with_reactions(
                        &Recipient::Team(team_id.clone()),
                        content,
                        reactions,
                    ));
                }
            }
        }
        message_ids
    }

    pub fn play_youtube_audio(&self, url: String) -> Result<LockedAudio> {
        let discord_output = self.discord_output.lock();
        discord_output.play_youtube_audio(self.guild_id, url)
    }

    pub fn play_file_audio(&self, path: &Path) -> Result<LockedAudio> {
        let discord_output = self.discord_output.lock();
        discord_output.play_file_audio(self.guild_id, path)
    }

    pub fn stop_audio(&self) -> Result<()> {
        let discord_output = self.discord_output.lock();
        discord_output.stop_audio(self.guild_id)
    }

    pub fn read_reactions(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<Vec<UserId>> {
        let discord_output = self.discord_output.lock();
        discord_output.read_reactions(channel_id, message_id, reaction)
    }
}
