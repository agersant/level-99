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
#[commands(guess, team)]
struct Main;

#[command]
fn guess(ctx: &mut SerenityContext, msg: &Message, args: Args) -> CommandResult {
    let result = || -> Result<()> {
        let game_pool = ctx
            .data
            .read()
            .get::<GamePool>()
            .cloned()
            .expect("Expected GamePool in ShareMap.");
        let game_lock = game_pool.get_game(ctx, msg.channel_id)?;
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
        game.join_team(msg.author.id, team_name)?;

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
