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

use crate::game::pool::Pool as GamePool;
use crate::VoiceManager;

#[group]
#[commands(join, begin, guess)]
struct General;

#[command]
fn begin(ctx: &mut SerenityContext, msg: &Message, args: Args) -> CommandResult {
    let result = || -> Result<()> {
        let manager = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected VoiceManager in ShareMap.");
        let game_lock = manager.get_game(ctx, msg.channel_id)?;
        let mut game = game_lock.lock();
        let path_string = args.parse::<String>().context("Filename cannot be blank")?;
        let path = Path::new(&path_string);
        game.begin(path)
            .with_context(|| format!("Could not begin quizz with path {:?}", path))?;
        Ok(())
    }();

    match result {
        Err(e) => {
            eprintln!("{:#}", e);
            Err(CommandError(e.to_string()))
        }
        Ok(_) => Ok(()),
    }
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
        game.guess(&guess)
            .with_context(|| format!("Could not process guess {}", guess))?;
        Ok(())
    }();

    match result {
        Err(e) => {
            eprintln!("{:#}", e);
            Err(CommandError(e.to_string()))
        }
        Ok(_) => Ok(()),
    }
}

#[command]
fn join(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

            return Ok(());
        }
    };

    let guild_id = guild.read().id;

    let channel_id = guild
        .read()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(&ctx, "Not in a voice channel"));
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

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        eprintln!("Error sending message: {:?}", why);
    }
}
