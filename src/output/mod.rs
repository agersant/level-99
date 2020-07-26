use anyhow::*;
use serenity::model::id::{ChannelId, MessageId, UserId};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use crate::game::quiz::definition::Question;
use crate::game::team::TeamId;

pub mod discord;
#[cfg(test)]
pub mod mock;

pub enum Recipient {
    AllTeams,
    Team(TeamId),
    AllTeamsExcept(TeamId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    AnswerReveal(Question),
    ChallengeSongBegins(String),
    ChallengeSongTimeUp(TeamId, i32),
    GamePaused,
    GameResults(TeamId),
    GameUnpaused,
    GuessCorrect(TeamId, i32),
    GuessesReveal(Vec<(TeamId, String)>),
    GuessIncorrect(TeamId, i32),
    QuizRules,
    PreloadFailed,
    ScoresRecap(Vec<(TeamId, i32)>),
    ScoresReset,
    QuestionBegins(Question),
    TeamScoreAdjusted(TeamId, i32),
    TeamsReset,
    TimeRemaining(Duration),
    TimeUp(Question),
    VotePoll(Vec<(String, String, u32)>),
    VoteWait(TeamId),
    WagerBegins(String),
    WagerResults(Vec<(TeamId, u32)>),
    WagerRules(u32, u32),
    WagerWait,
}

pub trait AudioHandle {
    fn is_finished(&self) -> bool;
}

pub trait GameOutput {
    type Audio: AudioHandle;

    fn say(
        &self,
        recipient: &Recipient,
        message: &Message,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>>;

    fn say_with_reactions(
        &self,
        recipient: &Recipient,
        message: &Message,
        reactions: &Vec<String>,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>>;

    fn play_youtube_audio(&self, url: String) -> Result<Self::Audio>;

    fn play_file_audio(&self, path: &Path) -> Result<Self::Audio>;

    fn stop_audio(&self) -> Result<()>;

    fn read_reactions(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<Vec<UserId>>;

    fn update_team_channels(&self, channel_ids: HashMap<TeamId, ChannelId>);
}
