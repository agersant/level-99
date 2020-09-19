use anyhow::*;
use serenity::{
    client::Context as SerenityContext,
    framework::standard::macros::{command, group},
    framework::standard::{Args, CommandError, CommandResult},
    model::channel::Message,
};

use crate::channels::*;
use crate::commands::*;
use crate::game::pool::Pool as GamePool;

#[group]
#[commands(guess, team, wager)]
struct Main;

#[command]
fn guess(ctx: &mut SerenityContext, msg: &Message, args: Args) -> CommandResult {
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

        let guess = args.rest();
        if guess.trim().len() != 0 {
            game.guess(msg.author.id, &guess)?;
        }
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

        let team_name = args.rest();
        game.join_team(msg.author.id, team_name)?;

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
fn wager(ctx: &mut SerenityContext, msg: &Message, mut args: Args) -> CommandResult {
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

        let amount = args.single()?;
        game.wager(msg.author.id, amount)?;
        Ok(())
    }();

    if let Err(e) = result {
        eprintln!("{:#}", e);
        check_msg(msg.reply(&ctx.http, format!("{}", e)));
        return Err(CommandError(e.to_string()));
    }
    Ok(())
}
