use serenity::{
    client::Context,
    framework::standard::macros::{command, group},
    framework::standard::CommandResult,
    model::channel::Message,
    model::misc::Mentionable,
    voice, Result as SerenityResult,
};

use crate::QuizzManager;
use crate::VoiceManager;

#[group]
#[commands(join, leave, question, answer)]
struct General;

#[command]
fn question(ctx: &mut Context, msg: &Message) -> CommandResult {
    let quizz_lock = ctx
        .data
        .read()
        .get::<QuizzManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut quizz = quizz_lock.lock();

    match quizz.begin_new_question() {
        Some(question) => {
            let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
                Some(channel) => channel.read().guild_id,
                None => {
                    check_msg(msg.channel_id.say(&ctx.http, "Error finding channel info"));

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
            if let Some(handler) = manager.get_mut(guild_id) {
                let source = match voice::ytdl(&question.url) {
                    Ok(source) => source,
                    Err(why) => {
                        println!("Err starting source: {:?}", why);
                        check_msg(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg"));
                        return Ok(());
                    }
                };
                handler.play_only(source);
                check_msg(msg.channel_id.say(&ctx.http, "Playing song"));
            } else {
                check_msg(
                    msg.channel_id
                        .say(&ctx.http, "Not in a voice channel to play in"),
                );
            }
        }
        None => {
            // TODO QUIZZ IS OVER
        }
    }

    Ok(())
}

#[command]
fn answer(ctx: &mut Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
fn join(ctx: &mut Context, msg: &Message) -> CommandResult {
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

#[command]
fn leave(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, "Groups and DMs not supported"),
            );

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
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel"));
    } else {
        check_msg(msg.reply(&ctx, "Not in a voice channel"));
    }

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
