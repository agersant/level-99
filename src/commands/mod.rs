use serenity::{model::channel::Message, Result as SerenityResult};
use std::path::Path;

pub mod player;
pub mod quizzmaster;

const ERROR_MISSING_GUILD: &'static str = "This command cannot be used in a group or DM.";

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        eprintln!("Error sending message: {:?}", why);
    }
}
