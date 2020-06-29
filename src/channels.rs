use anyhow::*;
use serenity::{
    client::Context as SerenityContext,
    model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType},
    model::id::{ChannelId, GuildId, RoleId},
    model::permissions::Permissions,
};
use std::collections::HashMap;

use crate::game::team::{Team, TeamId};

const TEAM_CHANNELS_CATEGORY: &'static str = "Team Channels";
const QUIZMASTER_ROLE: &'static str = "quizmaster";

pub fn update_team_channels(
    ctx: &SerenityContext,
    guild_id: GuildId,
    teams: &Vec<Team>,
) -> Result<HashMap<TeamId, ChannelId>> {
    // According to the docs on Guild.id: `This is equivalent to the Id of the default role (`@everyone`)`
    let everyone_role_id = RoleId::from(*guild_id.as_u64());
    let quizmaster_role_id = get_quizmaster_role_id(ctx, guild_id)?;

    // Make sure we have a category for team channels
    let channels = guild_id.channels(&ctx.http)?;
    let team_channel_category = channels.iter().find(|(_id, channel)| {
        channel.kind == ChannelType::Category && channel.name() == TEAM_CHANNELS_CATEGORY
    });
    let team_channel_category = match team_channel_category {
        Some((id, _channel)) => *id,
        None => {
            println!("Creating channel category: {}", TEAM_CHANNELS_CATEGORY);
            guild_id
                .create_channel(&ctx.http, |c| {
                    c.name(TEAM_CHANNELS_CATEGORY).kind(ChannelType::Category)
                })?
                .id
        }
    };

    // Remove team channels that have no team
    let channels = guild_id.channels(&ctx.http)?;
    for (_channel_id, channel) in &channels {
        if channel.category_id != Some(team_channel_category) {
            continue;
        }
        if teams
            .iter()
            .find(|t| t.get_display_name() == channel.name())
            .is_some()
        {
            continue;
        }
        println!("Deleting team channel: {}", channel.name());
        channel.delete(&ctx.http)?;
    }

    // Create missing team channels
    let mut channel_ids = HashMap::new();
    for team in teams {
        let channel_id = match channels.iter().find(|(_id, channel)| {
            channel.name() == team.get_display_name()
                && channel.category_id == Some(team_channel_category)
        }) {
            None => {
                println!("Creating team channel: {}", team.get_display_name());
                guild_id
                    .create_channel(&ctx.http, |c| {
                        c.name(team.get_display_name())
                            .category(team_channel_category)
                            .permissions(vec![PermissionOverwrite {
                                deny: Permissions::READ_MESSAGES,
                                allow: Permissions::empty(),
                                kind: PermissionOverwriteType::Role(everyone_role_id),
                            }])
                    })?
                    .id
            }
            Some((channel_id, _channel)) => *channel_id,
        };
        channel_ids.insert(team.id.clone(), channel_id);
    }

    // Adjust permissions on team channels
    let channels = guild_id.channels(&ctx.http)?;
    for (_channel_id, channel) in &channels {
        if channel.category_id != Some(team_channel_category) {
            continue;
        }
        if let Some(team) = teams
            .iter()
            .find(|t| t.get_display_name() == channel.name())
        {
            // Don't allow non-team members to read
            for permission in &channel.permission_overwrites {
                if let PermissionOverwriteType::Member(user_id) = permission.kind {
                    if !team.players.contains(&user_id) {
                        channel.delete_permission(&ctx.http, permission.kind)?;
                    }
                }
            }

            // Allow quizmaster to read
            channel.create_permission(
                &ctx.http,
                &PermissionOverwrite {
                    deny: Permissions::empty(),
                    allow: Permissions::READ_MESSAGES,
                    kind: PermissionOverwriteType::Role(quizmaster_role_id),
                },
            )?;

            // Allow team members to read
            for player in &team.players {
                let has_permission = channel
                    .permission_overwrites
                    .iter()
                    .find(|p| p.kind == PermissionOverwriteType::Member(*player))
                    .is_some();
                if !has_permission {
                    channel.create_permission(
                        &ctx.http,
                        &PermissionOverwrite {
                            deny: Permissions::empty(),
                            allow: Permissions::READ_MESSAGES,
                            kind: PermissionOverwriteType::Member(*player),
                        },
                    )?;
                }
            }
        }
    }

    Ok(channel_ids)
}

pub fn get_quizmaster_role_id(ctx: &SerenityContext, guild_id: GuildId) -> Result<RoleId> {
    let guild = guild_id.to_partial_guild(&ctx.http)?;
    match guild.role_by_name(QUIZMASTER_ROLE) {
        Some(r) => Ok(r.id),
        None => guild_id
            .create_role(&ctx.http, |r| r.hoist(true).name(QUIZMASTER_ROLE))
            .map(|r| r.id)
            .context("Could not create quizmaster role"),
    }
}
