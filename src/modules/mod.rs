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

mod auto_cleaner;
mod spam_filter;
pub mod terminal;

use crate::database::DmInbox;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref XP_FLUSHER_RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            tracing::info!("Connected as {}", data_about_bot.user.name);

            // Start auto-cleaner task
            auto_cleaner::start(ctx.clone(), data.db.clone()).await;

            // Start terminal
            terminal::start(ctx.clone(), data.db.clone()).await;

            // Start XP flusher
            start_xp_flusher(ctx.clone(), data.db.clone()).await;
        }

        serenity::FullEvent::Message { new_message } => {
            // Skip bot messages
            if new_message.author.bot {
                return Ok(());
            }

            // Check for bot-level ban
            if data.db.is_bot_banned(new_message.author.id.get()).await.unwrap_or(false) {
                return Ok(()); // Silently ignore banned users
            }

            // Handle DMs - save to inbox and respond
            if new_message.guild_id.is_none() {
                // Save DM to inbox
                let dm = DmInbox {
                    id: 0,
                    user_id: new_message.author.id.get(),
                    username: new_message.author.name.clone(),
                    content: new_message.content.clone(),
                    timestamp: chrono::Utc::now().timestamp(),
                    read_status: false,
                };

                if let Err(e) = data.db.save_dm(&dm).await {
                    tracing::error!("Failed to save DM: {}", e);
                }

                // Notify in console
                let preview = if new_message.content.len() > 50 {
                    format!("{}...", &new_message.content[..50])
                } else {
                    new_message.content.clone()
                };
                tracing::info!(
                    "New DM from {} ({}): {}",
                    new_message.author.name,
                    new_message.author.id,
                    preview
                );

                // Send auto-reply
                let dm_message = data
                    .config
                    .dm_message
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("I'm just a bot :'(. I can't answer to you.");

                let _ = new_message.channel_id.say(ctx, dm_message).await;
                return Ok(());
            }

            // Run spam filter
            if let Some(guild_id) = new_message.guild_id {
                spam_filter::process_message(ctx, new_message, guild_id, data).await?;

                // Add XP for chatting (batched)
                let xp_gain = rand::thread_rng().gen_range(15..26);
                data.db
                    .add_xp_to_batch(
                        new_message.author.id.get(),
                        guild_id.get(),
                        new_message.channel_id.get(),
                        xp_gain,
                    )
                    .await;
            }
        }

        serenity::FullEvent::GuildBanAddition { guild_id, banned_user } => {
            tracing::info!(
                "User {} was banned from guild {}",
                banned_user.name,
                guild_id
            );
        }

        serenity::FullEvent::Resume { .. } => {
            tracing::info!("Reconnected to Discord");
        }

        _ => {}
    }

    Ok(())
}

async fn start_xp_flusher(ctx: serenity::Context, db: crate::database::Database) {
    let mut running = XP_FLUSHER_RUNNING.lock().await;
    if *running {
        return;
    }
    *running = true;
    drop(running);

    let ctx = Arc::new(ctx);
    let db = Arc::new(db);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            interval.tick().await;

            match db.flush_xp_batch().await {
                Ok(level_ups) => {
                    for (user_id, _guild_id, channel_id, new_level, _new_xp) in level_ups {
                        let channel = serenity::ChannelId::new(channel_id);
                        let _ = channel
                            .say(
                                &*ctx,
                                format!(
                                    "**Level Up!** Congratulations <@{}>! You've reached level **{}**!",
                                    user_id, new_level
                                ),
                            )
                            .await;
                    }
                }
                Err(e) => {
                    tracing::error!("XP flusher error: {}", e);
                }
            }
        }
    });

    tracing::info!("XP flusher started");
}
