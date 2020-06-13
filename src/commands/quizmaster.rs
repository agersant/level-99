use anyhow::*;
use serenity::{
    client::Context as SerenityContext,
    framework::standard::macros::{check, command, group},
    framework::standard::{Args, CheckResult, CommandError, CommandResult},
    model::channel::Message,
    model::misc::Mentionable,
};

use crate::channels::*;
use crate::commands::*;
use crate::game::pool::Pool as GamePool;
use crate::game::team::TeamId;
use crate::VoiceManager;

const ERROR_USER_NOT_IN_VOICE: &'static str = "You must be in a voice channel to use this command.";
const ERROR_BOT_NOT_IN_VOICE: &'static str =
    "Use the `!join` command to invite the bot to a voice channel before starting the quiz.";

#[check]
#[name = "Quizmaster"]
fn quizmaster_check(ctx: &mut SerenityContext, msg: &Message) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }
    false.into()
}

#[group]
#[checks(Quizmaster)]
#[commands(begin, join, pause, score, skip, unpause)]
struct Main;

#[group]
#[checks(Quizmaster)]
#[prefix = "reset"]
#[commands(scores, teams)]
struct Reset;

#[command]
fn begin(ctx: &mut SerenityContext, msg: &Message, args: Args) -> CommandResult {
    let result = || -> Result<()> {
        let guild_id = msg
            .guild(&ctx.cache)
            .context(ERROR_MISSING_GUILD)?
            .read()
            .id;
        let voice_manager_lock = ctx
            .data
            .read()
            .get::<VoiceManager>()
            .cloned()
            .expect("Expected VoiceManager in ShareMap.");
        let voice_manager = voice_manager_lock.lock();
        if voice_manager.get(guild_id).is_none() {
            return Err(anyhow!(ERROR_BOT_NOT_IN_VOICE));
        }

        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, guild_id);
        let mut game = game_lock.lock();

        let path_string = args.rest();
        if path_string.is_empty() {
            return Err(anyhow!("Filename cannot be blank"));
        }
        let path = Path::new(&path_string);
        game.begin(path)
            .with_context(|| format!("Could not begin quiz with path {:?}", path))?;
        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

#[command]
fn join(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).context(ERROR_MISSING_GUILD)?;
    let guild_id = guild.read().id;

    let channel_id = guild
        .read()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(&ctx, ERROR_USER_NOT_IN_VOICE));
            return Ok(());
        }
    };

    let voice_manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = voice_manager_lock.lock();

    if manager.join(guild_id, connect_to).is_some() {
        check_msg(
            msg.channel_id
                .say(&ctx.http, &format!("Joined {}", connect_to.mention())),
        );
        check_msg(msg.channel_id.say(
            &ctx.http,
            "Use the `!team team-name` command to create or join a team",
        ));
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel"));
    }

    Ok(())
}

#[command]
fn pause(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let guild_id = ctx
        .cache
        .read()
        .guild_channel(msg.channel_id)
        .context("Server not found")?
        .read()
        .guild_id;
    let result = || -> Result<()> {
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, guild_id);
        let mut game = game_lock.lock();
        game.pause();
        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

#[command]
fn score(ctx: &mut SerenityContext, msg: &Message, mut args: Args) -> CommandResult {
    let result = || -> Result<()> {
        let guild_id = ctx
            .cache
            .read()
            .guild_channel(msg.channel_id)
            .context("Server not found")?
            .read()
            .guild_id;
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, guild_id);
        let mut game = game_lock.lock();

        let team_name = args
            .single::<String>()
            .context("Could not parse team name")?;
        let score_delta = args
            .single::<i32>()
            .context("Could not parse score delta")?;
        let team_id = TeamId::TeamName(team_name);
        game.adjust_score(team_id, score_delta)?;

        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

#[command]
fn skip(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let result = || -> Result<()> {
        let guild_id = ctx
            .cache
            .read()
            .guild_channel(msg.channel_id)
            .context("Server not found")?
            .read()
            .guild_id;
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, guild_id);
        let mut game = game_lock.lock();
        game.skip()?;
        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

#[command]
fn scores(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let guild_id = ctx
        .cache
        .read()
        .guild_channel(msg.channel_id)
        .context("Server not found")?
        .read()
        .guild_id;
    let result = || -> Result<()> {
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, guild_id);
        let mut game = game_lock.lock();
        game.reset_scores();
        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

#[command]
fn teams(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let result = || -> Result<()> {
        let guild_id = ctx
            .cache
            .read()
            .guild_channel(msg.channel_id)
            .context("Server not found")?
            .read()
            .guild_id;
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, guild_id);
        let mut game = game_lock.lock();
        game.reset_teams();

        let guild_id = msg
            .guild(&ctx.cache)
            .context(ERROR_MISSING_GUILD)?
            .read()
            .id;

        let channel_ids = update_team_channels(ctx, guild_id, &game.get_teams())?;
        game.update_team_channels(channel_ids);

        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

#[command]
fn unpause(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let result = || -> Result<()> {
        let guild_id = ctx
            .cache
            .read()
            .guild_channel(msg.channel_id)
            .context("Server not found")?
            .read()
            .guild_id;
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, guild_id);
        let mut game = game_lock.lock();
        game.unpause();
        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}
