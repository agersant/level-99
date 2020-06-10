use anyhow::*;
use serenity::{
    client::Context as SerenityContext,
    framework::standard::macros::{command, group},
    framework::standard::{CommandError, CommandResult},
    model::channel::Message,
};

use crate::channels::*;
use crate::commands::*;
use crate::game::pool::Pool as GamePool;

#[group]
#[prefix = "reset"]
#[commands(scores, teams)]
struct Reset;

#[command]
fn scores(ctx: &mut SerenityContext, msg: &Message) -> CommandResult {
    let result = || -> Result<()> {
        let manager = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected VoiceManager in ShareMap.");
        let game_lock = manager.get_game(ctx, msg.channel_id)?;
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
        let manager = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected VoiceManager in ShareMap.");
        let game_lock = manager.get_game(ctx, msg.channel_id)?;
        let mut game = game_lock.lock();
        game.reset_teams();

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
