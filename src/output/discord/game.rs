use anyhow::*;
use parking_lot::RwLock;
use serenity::model::id::{ChannelId, MessageId, UserId};
use serenity::voice::LockedAudio;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::game::team::{TeamId, TeamsHandle};
use crate::output::discord::GuildOutput;
use crate::output::{AudioHandle, GameOutput, Message, Recipient};

#[derive(Clone, Debug)]
pub struct DiscordGameOutput {
    guild_output: Arc<RwLock<GuildOutput>>,
    teams: TeamsHandle,
}

impl DiscordGameOutput {
    pub fn new(guild_output: GuildOutput, teams: TeamsHandle) -> Self {
        DiscordGameOutput {
            guild_output: Arc::new(RwLock::new(guild_output)),
            teams,
        }
    }

    fn get_team_display_name(&self, team_id: &TeamId) -> String {
        self.teams
            .read()
            .iter()
            .find(|t| t.id == *team_id)
            .and_then(|t| Some(t.get_display_name().to_owned()))
            .unwrap_or("??".to_owned())
    }

    fn interpret_message(&self, message: &Message) -> String {
        use Message::*;
        match message {
            TeamScoreAdjusted(team_id, score) => {
                let team_name = self.get_team_display_name(team_id);
                format!("Team {}'s score was updated to {} points", team_name, score)
            },
            TeamsReset=>"Teams were reset".into(),
            ScoresReset=>"Scores were reset".into(),
            GamePaused=>"The game is now paused, use `!unpause` to resume.".into(),
            GameUnpaused=>"The game has resumed.".into(),
            QuizRules => "The quiz is about to begin!\n\n**📋 Rules**\n- For each song, your team can submit **one** guess using the `!guess something` command.\n- Guessing wrong will deduct the same amount of points you could have earned!\n- If you are not the first team to guess, point earned or deducted are halved.\n\n**🔥 Tips**\n- Answers are game names, not song titles or composers.\n- You can adjust the music volume by right clicking on the bot in the voice channel UI.\n- Sometimes it is wiser to not answer than to lose points!".into(),
            GuessCorrect(team_id, score_delta) => {
                let team_name = self.get_team_display_name(team_id);
                format!("✅ **Team {}** guessed correctly and earned {} points!",team_name, score_delta)
            }
            GuessIncorrect(team_id, score_delta) => {
                let team_name = self.get_team_display_name(team_id);
                format!("❌ **Team {}** guessed incorrectly and lost {} points. Womp womp 📯.",team_name, score_delta)
            }
            AnswerReveal(question) => format!("The answer was **{}**:\n{}", question.answer, question.url),
            GuessesReveal(details) => {
                let mut message = "This is what everyone guessed:".to_owned();
                for (team_id, guess) in details {
                    message += &format!("\n- **Team {}**: {}", self.get_team_display_name(team_id), guess);
                }
                message
            },
            ScoresRecap(teams) => {
                let mut recap = "📈 Here are the scores so far:".to_owned();
                for (index, (team_id, score)) in teams.iter().enumerate() {
                    let rank = match index {
                        0 => "🥇".to_owned(),
                        1 => "🥈".to_owned(),
                        2 => "🥉".to_owned(),
                        _ => format!("#{}", index + 1),
                    };
                    recap += &format!(
                        "\n{} **Team {}** with {} points",
                        rank,
                        self.get_team_display_name(team_id),
                        score
                    );
                }
                recap
            }
            TimeRemaining(duration) => format!("🕒 Only {} seconds left!", duration.as_secs()),
            ChallengeSongBegins(category) => format!("🎧 Here is a song from the **{}** category! Your team **must** guess this one right or you will lose points.", category),
            SongBegins(category, value) => format!("🎧 Here is a song from the **{}** category for {} points!", category, value),
            TimeUp(question) => format!("⏰ Time's up! The answer was **{}**:\n{}", question.answer, question.url),
            ChallengeSongTimeUp(team_id, amount) => format!("**Team {}** loses *{} points* for not answering the **CHALLENGE** question!", self.get_team_display_name(team_id), amount),
            GameResults(team_id) => format!("🎊🎊 **TEAM {} WINS IT ALL!** 🎊🎊", self.get_team_display_name(team_id)).to_uppercase(),
            VoteWait(team_id) => format!("⏳ **Team {}** is choosing a category for the next question.", self.get_team_display_name(team_id)),
            VotePoll(options) => {
                let mut message = "**🗳️ Choose a category**\nReact to this message to cast your vote for the next question's category!".to_owned();
                for (reaction, category, value) in options {
                    message += &format!("\n{} **{}** {}pts", reaction, category, value);
                }
                message
            }  ,
            WagerBegins(category) => format!("⚠️ A **CHALLENGE** question has appeared in the **{}** category!", category),
            WagerWait => "⏳ Please wait while other teams are responding to the **CHALLENGE** question.".into(),
            WagerRules(min, max) => format!("🍀 **Your team must answer this question**. Use the `!wager amount` command to wager between {} and {} points. This is the amount your team will earn or lose from this question.", min, max),
            WagerResults(wagers) => {
                let mut message = String::new();
                for (team_id, amount) in wagers {
                    message += &format!("**Team {}** is betting *{} points*!\n", self.get_team_display_name(team_id), amount);
                }
                message
            }
        }
    }
}

pub struct DiscordAudio {
    locked_audio: LockedAudio,
}

impl DiscordAudio {
    pub fn new(locked_audio: LockedAudio) -> Self {
        Self { locked_audio }
    }
}

impl AudioHandle for DiscordAudio {
    fn is_finished(&self) -> bool {
        self.locked_audio.lock().finished
    }
}

impl GameOutput for DiscordGameOutput {
    type Audio = DiscordAudio;

    fn say(
        &self,
        recipient: &Recipient,
        message: &Message,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let content = self.interpret_message(message);
        self.guild_output.read().say(recipient, &content)
    }

    fn say_with_reactions(
        &self,
        recipient: &Recipient,
        message: &Message,
        reactions: &Vec<String>,
    ) -> HashMap<TeamId, Result<(ChannelId, MessageId)>> {
        let content = self.interpret_message(message);
        self.guild_output
            .read()
            .say_with_reactions(recipient, &content, reactions)
    }

    fn play_youtube_audio(&self, url: String) -> Result<DiscordAudio> {
        self.guild_output
            .read()
            .play_youtube_audio(url)
            .map(DiscordAudio::new)
    }

    fn play_file_audio(&self, path: &Path) -> Result<DiscordAudio> {
        self.guild_output
            .read()
            .play_file_audio(path)
            .map(DiscordAudio::new)
    }

    fn stop_audio(&self) -> Result<()> {
        self.guild_output.read().stop_audio()
    }

    fn read_reactions(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        reaction: String,
    ) -> Result<Vec<UserId>> {
        self.guild_output
            .read()
            .read_reactions(channel_id, message_id, reaction)
    }

    fn update_team_channels(&self, channel_ids: HashMap<TeamId, ChannelId>) {
        self.guild_output.write().update_team_channels(channel_ids)
    }
}
