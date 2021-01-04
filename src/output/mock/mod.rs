use anyhow::*;
use parking_lot::RwLock;
use serenity::model::id::{ChannelId, MessageId, UserId};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::game::team::{TeamId, TeamsHandle};
use crate::output::{AudioHandle, GameOutput, Message, Recipient};

#[derive(Clone, Eq, PartialEq)]
pub struct TextEntry {
    pub message: Message,
    pub message_id: MessageId,
    pub reactions: HashMap<String, HashSet<UserId>>,
}

#[derive(Clone)]
pub struct MockGameOutput {
    text_output: Arc<RwLock<HashMap<TeamId, Vec<TextEntry>>>>,
    audio_output: Arc<RwLock<Option<MockAudio>>>,
    teams: TeamsHandle,
    message_count: u64,
}

impl MockGameOutput {
    pub fn new(teams: TeamsHandle) -> Self {
        let mut text_channels = HashMap::new();
        for team in teams.read().iter() {
            text_channels.insert(team.id.clone(), vec![]);
        }
        Self {
            text_output: Arc::new(RwLock::new(text_channels)),
            audio_output: Arc::new(RwLock::new(None)),
            teams,
            message_count: 0,
        }
    }

    pub fn read_channel(&mut self, team_id: &TeamId) -> Vec<TextEntry> {
        self.text_output
            .read()
            .get(team_id)
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    pub fn contains_message(&self, team_id: &TeamId, message: &Message) -> bool {
        self.text_output
            .read()
            .get(team_id)
            .unwrap()
            .iter()
            .any(|text_entry| match text_entry {
                TextEntry {
                    message: m,
                    message_id: _,
                    reactions: _,
                } if m == message => true,
                _ => false,
            })
    }

    pub fn is_playing_audio(&self, path: &Path) -> bool {
        match self.audio_output.read().deref() {
            Some(mock_audio) => mock_audio.source.as_path() == path,
            None => false,
        }
    }

    fn next_message_id(&mut self) -> MessageId {
        self.message_count += 1;
        MessageId(self.message_count)
    }

    pub fn react_to_message(&mut self, message_id: MessageId, reaction: String, user_id: UserId) {
        for channel in self.text_output.write().values_mut() {
            if let Some(text_entry) = channel
                .iter_mut()
                .find(|text_entry| text_entry.message_id == message_id)
            {
                if let Some(reactions) = text_entry.reactions.get_mut(&reaction) {
                    reactions.insert(user_id);
                } else {
                    let mut users = HashSet::new();
                    users.insert(user_id);
                    text_entry.reactions.insert(reaction.clone(), users);
                }
            }
        }
    }

    fn recipient_to_team_ids(&self, recipient: &Recipient) -> Vec<TeamId> {
        match recipient {
            Recipient::Team(id) => vec![id.clone()],
            Recipient::AllTeams => self.teams.read().iter().map(|t| t.id.clone()).collect(),
            Recipient::AllTeamsExcept(team_id) => self
                .teams
                .read()
                .iter()
                .map(|t| t.id.clone())
                .filter(|id| id != team_id)
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct MockAudio {
    pub source: PathBuf,
}

impl MockAudio {
    fn new(source: PathBuf) -> Self {
        Self { source }
    }
}

impl AudioHandle for MockAudio {
    fn is_finished(&self) -> bool {
        false
    }
}

impl GameOutput for MockGameOutput {
    type Audio = MockAudio;

    fn say(
        &mut self,
        recipient: &Recipient,
        message: &Message,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let mut output = HashMap::new();

        for id in &self.recipient_to_team_ids(recipient) {
            let message_id = self.next_message_id();
            self.text_output
                .write()
                .get_mut(id)
                .unwrap()
                .push(TextEntry {
                    message: message.clone(),
                    message_id: message_id,
                    reactions: HashMap::new(),
                });
            output.insert(id.clone(), Ok((ChannelId(0), message_id)));
        }

        output
    }

    fn say_with_reactions(
        &mut self,
        recipient: &Recipient,
        message: &Message,
        _reactions: &Vec<String>,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let mut output = HashMap::new();

        for id in &self.recipient_to_team_ids(recipient) {
            let message_id = self.next_message_id();
            self.text_output
                .write()
                .get_mut(id)
                .unwrap()
                .push(TextEntry {
                    message: message.clone(),
                    message_id: message_id,
                    reactions: HashMap::new(),
                });
            output.insert(id.clone(), Ok((ChannelId(0), message_id)));
        }

        output
    }

    fn play_youtube_audio(&self, _url: String) -> Result<MockAudio> {
        Ok(MockAudio::new(PathBuf::new()))
    }

    fn play_file_audio(&self, path: &Path) -> Result<MockAudio> {
        let mock_audio = MockAudio::new(path.to_path_buf());
        *self.audio_output.write() = Some(mock_audio.clone());
        Ok(mock_audio)
    }

    fn stop_audio(&self) -> Result<()> {
        *self.audio_output.write() = None;
        Ok(())
    }

    fn read_reactions(
        &self,
        _channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<HashSet<UserId>> {
        for (id, channel) in self.text_output.read().iter() {
            if let Some(text_entry) = channel
                .iter()
                .find(|text_entry| text_entry.message_id == message_id)
            {
                return Ok(text_entry
                    .reactions
                    .get(&reaction)
                    .cloned()
                    .unwrap_or(HashSet::new()));
            }
        }

        Ok(HashSet::new())
    }

    fn update_team_channels(&self, _channel_ids: HashMap<TeamId, ChannelId>) {}
}
