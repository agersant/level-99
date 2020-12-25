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

#[derive(PartialEq, Eq)]
pub struct TextEntry {
    pub message: Message,
    pub message_id: MessageId,
}

#[derive(Clone)]
pub struct MockGameOutput {
    text_output: Arc<RwLock<HashMap<TeamId, Vec<TextEntry>>>>,
    audio_output: Arc<RwLock<Option<MockAudio>>>,
    reactions: Arc<RwLock<HashMap<MessageId, HashMap<String, HashSet<UserId>>>>>,
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
            reactions: Arc::new(RwLock::new(HashMap::new())),
            teams,
            message_count: 0,
        }
    }

    pub fn flush(&mut self, team_id: &TeamId) -> Vec<TextEntry> {
        std::mem::replace(
            self.text_output.write().get_mut(team_id).unwrap(),
            Vec::new(),
        )
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

    fn react_to_message(&mut self, message_id: MessageId, reaction: String, user_id: UserId) {
        let mut reactions = self.reactions.write();
        if reactions.contains_key(&message_id) {
            if reactions.get(&message_id).unwrap().contains_key(&reaction) {
                reactions
                    .get_mut(&message_id)
                    .unwrap()
                    .get_mut(&reaction)
                    .unwrap()
                    .insert(user_id);
            } else {
                let mut user_ids = HashSet::new();
                user_ids.insert(user_id);
                reactions
                    .get_mut(&message_id)
                    .unwrap()
                    .insert(reaction, user_ids);
            }
        } else {
            let mut new_reactions = HashMap::new();
            let mut user_ids = HashSet::new();
            user_ids.insert(user_id);
            new_reactions.insert(reaction, user_ids);
            reactions.insert(message_id, new_reactions);
        }
    }
}

pub struct MockAudio {
    pub source: PathBuf,
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
        _recipient: &Recipient,
        message: &Message,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let message_id = self.next_message_id();
        self.entries.write().push(Entry::Text(TextEntry {
            message: message.clone(),
            message_id,
        }));
        HashMap::new()
    }

    fn say_with_reactions(
        &mut self,
        recipient: &Recipient,
        message: &Message,
        reactions: &Vec<String>,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let mut output = HashMap::new();
        let team_ids: Vec<TeamId> = match recipient {
            Recipient::Team(id) => vec![id.clone()],
            Recipient::AllTeams => self.teams.read().iter().map(|t| t.id.clone()).collect(),
            Recipient::AllTeamsExcept(team_id) => self
                .teams
                .read()
                .iter()
                .map(|t| t.id.clone())
                .filter(|id| id != team_id)
                .collect(),
        };

        for id in team_ids {
            let message_id = self.next_message_id();
            self.entries.write().push(Entry::Text(TextEntry {
                message: message.clone(),
                message_id: message_id,
            }));
            output.insert(id, Ok((ChannelId(0), message_id)));
        }

        output
    }

    fn play_youtube_audio(&self, _url: String) -> Result<MockAudio> {
        Ok(MockAudio {})
    }

    fn play_file_audio(&self, path: &Path) -> Result<MockAudio> {
        self.entries.write().push(Entry::Audio(path.to_path_buf()));
        Ok(MockAudio {})
    }

    fn stop_audio(&self) -> Result<()> {
        Ok(())
    }

    fn read_reactions(
        &self,
        _channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<Vec<UserId>> {
        Ok(self
            .reactions
            .write()
            .get(&message_id)
            .and_then(|r| r.get(&reaction))
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or(vec![]))
    }

    fn update_team_channels(&self, _channel_ids: HashMap<TeamId, ChannelId>) {}
}
