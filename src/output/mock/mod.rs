use anyhow::*;
use parking_lot::RwLock;
use serenity::model::id::{ChannelId, MessageId, UserId};
use std::path::Path;
use std::sync::Arc;

use std::collections::HashMap;

use crate::game::team::TeamId;
use crate::output::{AudioHandle, GameOutput, Message, Recipient};

#[derive(Clone)]
pub struct MockGameOutput {
    messages: Arc<RwLock<Vec<Message>>>,
}

impl MockGameOutput {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn flush(&mut self) -> Vec<Message> {
        std::mem::replace(self.messages.write().as_mut(), Vec::new())
    }
}

pub struct MockAudio {}

impl AudioHandle for MockAudio {
    fn is_finished(&self) -> bool {
        false
    }
}

impl GameOutput for MockGameOutput {
    type Audio = MockAudio;

    fn say(
        &self,
        _recipient: &Recipient,
        message: &Message,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        self.messages.write().push(message.clone());
        HashMap::new()
    }

    fn say_with_reactions(
        &self,
        _recipient: &Recipient,
        _message: &Message,
        _reactions: &Vec<String>,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        HashMap::new()
    }

    fn play_youtube_audio(&self, _url: String) -> Result<MockAudio> {
        Ok(MockAudio {})
    }

    fn play_file_audio(&self, _path: &Path) -> Result<MockAudio> {
        Ok(MockAudio {})
    }

    fn stop_audio(&self) -> Result<()> {
        Ok(())
    }

    fn read_reactions(
        &self,
        _channel_id: ChannelId,
        _message_id: MessageId,
        _reaction: String,
    ) -> Result<Vec<UserId>> {
        Ok(Vec::new())
    }

    fn update_team_channels(&self, _channel_ids: HashMap<TeamId, ChannelId>) {}
}
