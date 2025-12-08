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
use std::collections::HashMap;

/// Ban a user from the server
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "BAN_MEMBERS",
    guild_only
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Reason for the ban"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let reason_str = reason.clone().unwrap_or_else(|| format!("Banned by {}", ctx.author().name));

    // Ban the user
    guild_id
        .ban_with_reason(ctx.http(), user.id, 1, &reason_str)
        .await?;

    // Record to database
    ctx.data()
        .db
        .add_mod_action(
            guild_id.get(),
            ctx.author().id.get(),
            user.id.get(),
            "ban",
            reason.as_deref(),
            chrono::Utc::now().timestamp(),
        )
        .await?;

    ctx.say(format!(
        ":white_check_mark: Successfully banned **{}** ({})",
        user.name, user.id
    ))
    .await?;

    Ok(())
}

/// Kick a user from the server
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "KICK_MEMBERS",
    guild_only
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "User to kick"] user: serenity::User,
    #[description = "Reason for the kick"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let reason_str = reason.clone().unwrap_or_else(|| format!("Kicked by {}", ctx.author().name));

    // Get the member
    let member = guild_id.member(ctx.http(), user.id).await?;

    // Kick the user
    member.kick_with_reason(ctx.http(), &reason_str).await?;

    // Record to database
    ctx.data()
        .db
        .add_mod_action(
            guild_id.get(),
            ctx.author().id.get(),
            user.id.get(),
            "kick",
            reason.as_deref(),
            chrono::Utc::now().timestamp(),
        )
        .await?;

    ctx.say(format!(
        ":white_check_mark: Successfully kicked **{}** ({})",
        user.name, user.id
    ))
    .await?;

    Ok(())
}

/// Unban a user from the server
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "BAN_MEMBERS",
    guild_only
)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "User ID to unban"] user_id: String,
    #[description = "Reason for the unban"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let user_id: u64 = user_id.parse().map_err(|_| "Invalid user ID")?;

    // Unban the user
    guild_id.unban(ctx.http(), serenity::UserId::new(user_id)).await?;

    // Record to database
    ctx.data()
        .db
        .add_mod_action(
            guild_id.get(),
            ctx.author().id.get(),
            user_id,
            "unban",
            reason.as_deref(),
            chrono::Utc::now().timestamp(),
        )
        .await?;

    ctx.say(format!(":white_check_mark: Successfully unbanned user {}", user_id))
        .await?;

    Ok(())
}

/// Timeout a user
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MODERATE_MEMBERS",
    guild_only
)]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "User to timeout"] user: serenity::User,
    #[description = "Duration in minutes"] duration: i64,
    #[description = "Reason for the timeout"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;

    if duration < 1 || duration > 40320 {
        ctx.say(":x: Duration must be between 1 and 40320 minutes (28 days)")
            .await?;
        return Ok(());
    }

    let mut member = guild_id.member(ctx.http(), user.id).await?;

    // Calculate timeout end time
    let timeout_until = chrono::Utc::now() + chrono::Duration::minutes(duration);

    // Apply timeout
    member
        .disable_communication_until_datetime(ctx.http(), timeout_until.into())
        .await?;

    // Record to database
    ctx.data()
        .db
        .add_mod_action(
            guild_id.get(),
            ctx.author().id.get(),
            user.id.get(),
            "timeout",
            reason.as_deref(),
            chrono::Utc::now().timestamp(),
        )
        .await?;

    ctx.say(format!(
        ":white_check_mark: Timed out **{}** for {} minutes",
        user.name, duration
    ))
    .await?;

    Ok(())
}

/// Clean messages from a channel
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_MESSAGES",
    guild_only
)]
pub async fn clean(
    ctx: Context<'_>,
    #[description = "Number of messages to delete (max 100)"] count: Option<u8>,
) -> Result<(), Error> {
    let count = count.unwrap_or(100).min(100);

    let channel = ctx.channel_id();
    let messages = channel
        .messages(ctx.http(), serenity::GetMessages::new().limit(count))
        .await?;

    let message_ids: Vec<_> = messages.iter().map(|m| m.id).collect();

    if message_ids.len() > 1 {
        channel.delete_messages(ctx.http(), &message_ids).await?;
    } else if let Some(msg) = messages.first() {
        msg.delete(ctx.http()).await?;
    }

    let reply = ctx
        .say(format!(":white_check_mark: Deleted {} messages", message_ids.len()))
        .await?;

    // Delete the confirmation after 3 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    reply.delete(ctx).await?;

    Ok(())
}

/// Show moderator statistics
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_ROLES",
    guild_only
)]
pub async fn mod_stats(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let guild = guild_id.to_partial_guild(ctx.http()).await?;

    ctx.defer().await?;

    // Get stats from database
    let stats = ctx.data().db.get_mod_stats(guild_id.get()).await?;

    // Build action totals
    let mut action_totals: HashMap<String, i64> = HashMap::new();
    for (action, count) in &stats.action_counts {
        action_totals.insert(action.clone(), *count);
    }

    // Build embed
    let mut embed = serenity::CreateEmbed::new()
        .title(format!("Moderator Statistics for {}", guild.name))
        .color(0xff69b4)
        .timestamp(chrono::Utc::now());

    // Add overview
    embed = embed.field(
        "Overview",
        format!(
            "**Total Tracked Actions:** {}",
            stats.total
        ),
        false,
    );

    // Add action breakdown
    if stats.total > 0 {
        embed = embed.field(
            "Action Breakdown",
            format!(
                "**Bans:** {}\n**Unbans:** {}\n**Kicks:** {}\n**Timeouts:** {}",
                action_totals.get("ban").unwrap_or(&0),
                action_totals.get("unban").unwrap_or(&0),
                action_totals.get("kick").unwrap_or(&0),
                action_totals.get("timeout").unwrap_or(&0),
            ),
            true,
        );
    }

    // Add top moderators
    if !stats.top_mods.is_empty() {
        let mut top_mods_text = String::new();
        for (i, (mod_id, count)) in stats.top_mods.iter().take(5).enumerate() {
            let mod_name = match mod_id.parse::<u64>() {
                Ok(id) => {
                    match serenity::UserId::new(id).to_user(ctx.http()).await {
                        Ok(user) => user.name,
                        Err(_) => mod_id.clone(),
                    }
                }
                Err(_) => mod_id.clone(),
            };
            top_mods_text.push_str(&format!("{}. **{}** - {} actions\n", i + 1, mod_name, count));
        }
        embed = embed.field("Top Moderators", top_mods_text, false);
    } else {
        embed = embed.field(
            "No Tracked Actions",
            "Run `/scan-bans` to import moderation history.",
            false,
        );
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

/// Import moderation actions from ban list or audit logs
#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "BAN_MEMBERS",
    guild_only
)]
pub async fn scan_bans(
    ctx: Context<'_>,
    #[description = "Scan mode: 'bans' for full ban list, 'audit' for audit logs"]
    mode: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("Must be used in a guild")?;
    let mode = mode.unwrap_or_else(|| "bans".to_string());

    ctx.defer().await?;

    let status_msg = ctx.say(":hourglass: Scanning... This may take a while.").await?;

    match mode.as_str() {
        "bans" | "banlist" => {
            // Fetch all bans
            let bans = guild_id.bans(ctx.http(), None, None).await?;
            let mut imported = 0;

            for ban in bans {
                // Add to database (we don't know who banned, so moderator is "unknown")
                ctx.data()
                    .db
                    .add_mod_action(
                        guild_id.get(),
                        0, // Unknown moderator
                        ban.user.id.get(),
                        "ban",
                        ban.reason.as_deref(),
                        chrono::Utc::now().timestamp(),
                    )
                    .await?;
                imported += 1;
            }

            status_msg
                .edit(ctx, poise::CreateReply::default().content(format!(
                    ":white_check_mark: Imported {} bans from ban list.\nNote: Moderator info is not available from ban list.",
                    imported
                )))
                .await?;
        }
        "audit" | "auditlog" => {
            status_msg
                .edit(ctx, poise::CreateReply::default().content(
                    ":information_source: Audit log scanning requires additional API calls. Use `/scan-bans bans` for basic import.",
                ))
                .await?;
        }
        _ => {
            status_msg
                .edit(ctx, poise::CreateReply::default().content(
                    "Usage: `/scan-bans [bans|audit]`\n- `bans`: Import from ban list\n- `audit`: Import from audit logs (limited history)",
                ))
                .await?;
        }
    }

    Ok(())
}
