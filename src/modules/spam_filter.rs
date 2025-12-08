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

use crate::Data;
use poise::serenity_prelude as serenity;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;

static DISCORD_INVITE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(https)*(http)*:*(//)*(discord(\.gg|app\.com/invite)/[a-zA-Z0-9]+)").unwrap()
});

static LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(ftp|http|https)://[^\s]+").unwrap()
});

static WARNINGS: LazyLock<Mutex<HashMap<u64, u32>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn process_message(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get member permissions
    let member = match guild_id.member(ctx, msg.author.id).await {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    // Skip if user can manage messages (moderator)
    if member.permissions(ctx)?.manage_messages() {
        return Ok(());
    }

    let content = &msg.content;

    // Check for @everyone/@here
    if content.contains("@everyone") || content.contains("@here") {
        return handle_violation(ctx, msg, &member, "Usage of @everyone/@here", data).await;
    }

    // Check for Discord invite links
    if DISCORD_INVITE_REGEX.is_match(content) {
        return handle_violation(ctx, msg, &member, "Discord invitation link", data).await;
    }

    // Check for any links
    if LINK_REGEX.is_match(content) {
        return handle_violation(ctx, msg, &member, "Link sent", data).await;
    }

    Ok(())
}

async fn handle_violation(
    ctx: &serenity::Context,
    msg: &serenity::Message,
    member: &serenity::Member,
    reason: &str,
    data: &Data,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let max_warnings = data.config.spam_max_warnings;
    let user_id = msg.author.id.get();

    // Delete the message
    let _ = msg.delete(ctx).await;

    // Get current warnings
    let mut warnings = WARNINGS.lock().await;
    let count = warnings.entry(user_id).or_insert(0);
    *count += 1;

    if *count >= max_warnings {
        // Ban the user
        warnings.remove(&user_id);

        let _ = msg.author.dm(ctx, serenity::CreateMessage::new().embed(
            serenity::CreateEmbed::new()
                .title("And here you go. You got banned!")
                .description(format!("Reason: {}", reason))
                .color(0xff0000)
        )).await;

        member
            .ban_with_reason(ctx, 1, format!("Autobanned by spam filter: {}. Used all warnings.", reason))
            .await?;

        // Record to database
        data.db
            .add_mod_action(
                member.guild_id.get(),
                ctx.cache.current_user().id.get(),
                user_id,
                "ban",
                Some(&format!("Autobanned: {}", reason)),
                chrono::Utc::now().timestamp(),
            )
            .await?;
    } else {
        // Send warning
        let _ = msg.author.dm(ctx, serenity::CreateMessage::new().embed(
            serenity::CreateEmbed::new()
                .title("Be careful! You're getting warned!")
                .description(format!(
                    "{}\nYou have {} warning(s). You'll be banned at {} warning(s).",
                    reason, count, max_warnings
                ))
        )).await;
    }

    Ok(())
}
