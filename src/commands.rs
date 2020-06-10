use anyhow::*;
use serenity::{
    client::Context as SerenityContext,
    framework::standard::macros::{command, group},
    framework::standard::{Args, CommandError, CommandResult},
    model::channel::Message,
    model::misc::Mentionable,
    Result as SerenityResult,
};
use std::path::Path;

use crate::channels::*;
use crate::game::pool::Pool as GamePool;
use crate::game::team::TeamId;
use crate::VoiceManager;

#[group]
#[commands(begin, guess, join, score, skip, team)]
struct General;

const ERROR_MISSING_GUILD: &'static str = "This command cannot be used in a group or DM.";
const ERROR_USER_NOT_IN_VOICE: &'static str = "You must be in a voice channel to use this command.";
const ERROR_BOT_NOT_IN_VOICE: &'static str =
    "Use the `!join` command to invite the bot to a voice channel before starting the quizz.";

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
        let game_lock = game_pool.get_game(ctx, msg.channel_id)?;
        let mut game = game_lock.lock();

        let path_string = args.parse::<String>().context("Filename cannot be blank")?;
        let path = Path::new(&path_string);
        game.begin(path)
            .with_context(|| format!("Could not begin quizz with path {:?}", path))?;
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
fn guess(ctx: &mut SerenityContext, msg: &Message, args: Args) -> CommandResult {
    let result = || -> Result<()> {
        let manager = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected VoiceManager in ShareMap.");
        let game_lock = manager.get_game(ctx, msg.channel_id)?;
        let mut game = game_lock.lock();

        let guess = args.rest();
        game.guess(msg.author.id, &guess)?;
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

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if manager.join(guild_id, connect_to).is_some() {
        check_msg(
            msg.channel_id
                .say(&ctx.http, &format!("Joined {}", connect_to.mention())),
        );
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "Error joining the channel"));
    }

    Ok(())
}

#[command]
fn score(ctx: &mut SerenityContext, msg: &Message, mut args: Args) -> CommandResult {
    let result = || -> Result<()> {
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, msg.channel_id)?;
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
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, msg.channel_id)?;
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
fn team(ctx: &mut SerenityContext, msg: &Message, args: Args) -> CommandResult {
    let result = || -> Result<()> {
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, msg.channel_id)?;
        let mut game = game_lock.lock();

        let team_name = args.rest();
        // TODO Trim, and only keep letters and numbers. Replace everything else with hyphens.
        if !team_name.is_empty() {
            game.join_team(msg.author.id, team_name)?;
        }

        let guild_id = msg
            .guild(&ctx.cache)
            .context(ERROR_MISSING_GUILD)?
            .read()
            .id;

        update_team_channels(ctx, guild_id, &game.get_teams())?;

        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}

// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        eprintln!("Error sending message: {:?}", why);
    }
}
