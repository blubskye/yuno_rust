/*
    Yuno Gasai - A Discord bot with moderation, auto-cleaning, and utility features.
    Copyright (C) 2018 Maeeen <maeeennn@gmail.com>
    Copyright (C) 2025 blubskye

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Check the bot's latency
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = std::time::Instant::now();
    let msg = ctx.say(":ping_pong: Pinging...").await?;
    let elapsed = start.elapsed();

    msg.edit(ctx, poise::CreateReply::default().content(format!(
        ":ping_pong: Pong! Latency: {}ms",
        elapsed.as_millis()
    )))
    .await?;

    Ok(())
}

/// Show help for commands
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command to get help for"] command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration::default(),
    )
    .await?;
    Ok(())
}

/// Set the bot prefix for this server
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_GUILD",
    guild_only
)]
pub async fn prefix(
    ctx: Context<'_>,
    #[description = "New prefix for the bot"] new_prefix: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;

    if new_prefix.len() > 10 {
        ctx.say(":x: Prefix must be 10 characters or less").await?;
        return Ok(());
    }

    ctx.data().db.set_prefix(guild_id.get(), &new_prefix).await?;

    ctx.say(format!(":white_check_mark: Prefix set to `{}`", new_prefix))
        .await?;

    Ok(())
}

/// Delay the auto-clean for this channel
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn delay(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let channel = ctx.channel_id().to_channel(ctx.http()).await?;

    let channel_name = match &channel {
        serenity::Channel::Guild(gc) => gc.name.clone(),
        _ => return Err("Must be used in a guild channel".into()),
    };

    // Check if auto-clean is set up
    let clean = ctx.data().db.get_clean(guild_id.get(), &channel_name).await?;

    match clean {
        Some(config) => {
            let new_remaining = config.remaining_time + 5; // Add 5 minutes
            ctx.data()
                .db
                .set_clean(
                    guild_id.get(),
                    &channel_name,
                    config.time_between_cleans,
                    config.warning_time,
                    Some(new_remaining),
                )
                .await?;

            ctx.say(format!(
                ":white_check_mark: Delayed auto-clean by 5 minutes. New time until clean: {} minutes",
                new_remaining
            ))
            .await?;
        }
        None => {
            ctx.say(":x: This channel doesn't have auto-clean set up.")
                .await?;
        }
    }

    Ok(())
}

/// Manage auto-clean settings for channels
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only,
    subcommands("auto_clean_add", "auto_clean_remove", "auto_clean_list")
)]
pub async fn auto_clean(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add auto-clean to a channel
#[poise::command(slash_command, prefix_command, rename = "add")]
pub async fn auto_clean_add(
    ctx: Context<'_>,
    #[description = "Channel to add auto-clean to"] channel: serenity::GuildChannel,
    #[description = "Hours between each clean"] hours: i64,
    #[description = "Minutes before clean to show warning"] warning_minutes: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;

    if hours < 1 {
        ctx.say(":x: Hours must be at least 1").await?;
        return Ok(());
    }

    if warning_minutes < 1 || warning_minutes >= hours * 60 {
        ctx.say(":x: Warning minutes must be at least 1 and less than the total clean time")
            .await?;
        return Ok(());
    }

    ctx.data()
        .db
        .set_clean(
            guild_id.get(),
            &channel.name,
            hours,
            warning_minutes,
            None,
        )
        .await?;

    ctx.say(format!(
        ":white_check_mark: Auto-clean added for {}. Will clean every {} hour(s) with a warning {} minute(s) before.",
        channel.name, hours, warning_minutes
    ))
    .await?;

    Ok(())
}

/// Remove auto-clean from a channel
#[poise::command(slash_command, prefix_command, rename = "remove")]
pub async fn auto_clean_remove(
    ctx: Context<'_>,
    #[description = "Channel to remove auto-clean from"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;

    ctx.data().db.delete_clean(guild_id.get(), &channel.name).await?;

    ctx.say(format!(
        ":white_check_mark: Auto-clean removed from {}",
        channel.name
    ))
    .await?;

    Ok(())
}

/// List auto-clean settings
#[poise::command(slash_command, prefix_command, rename = "list")]
pub async fn auto_clean_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;

    let cleans = ctx.data().db.get_all_cleans().await?;
    let guild_cleans: Vec<_> = cleans
        .iter()
        .filter(|c| c.guild_id == guild_id.get())
        .collect();

    if guild_cleans.is_empty() {
        ctx.say("No auto-cleans configured for this server.").await?;
    } else {
        let mut message = String::from("**Auto-clean configurations:**\n");
        for clean in guild_cleans {
            message.push_str(&format!(
                "• **#{}** - Every {} hour(s), warning at {} min, {} min remaining\n",
                clean.channel_name,
                clean.time_between_cleans,
                clean.warning_time,
                clean.remaining_time
            ));
        }
        ctx.say(message).await?;
    }

    Ok(())
}

/// Check your XP and level
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn xp(
    ctx: Context<'_>,
    #[description = "User to check XP for"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let target = user.as_ref().unwrap_or_else(|| ctx.author());

    let (xp, level) = ctx
        .data()
        .db
        .get_xp(guild_id.get(), target.id.get())
        .await?;

    let embed = serenity::CreateEmbed::new()
        .title(format!("{}'s XP", target.name))
        .color(0xff69b4)
        .thumbnail(target.face())
        .field("Level", level.to_string(), true)
        .field("XP", xp.to_string(), true);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Get the source code for this bot
#[poise::command(slash_command, prefix_command)]
pub async fn source(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(
        "**Yuno Gasai** is open source under AGPL-3.0!\n\
        Rust version: <https://github.com/blubskye/yuno_rust>\n\
        Original JS version: <https://github.com/japaneseenrichmentorganization/Yuno-Gasai-2>",
    )
    .await?;
    Ok(())
}
