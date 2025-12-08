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

use crate::database::Database;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref CLEANER_RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

pub async fn start(ctx: serenity::Context, db: Database) {
    let mut running = CLEANER_RUNNING.lock().await;
    if *running {
        return;
    }
    *running = true;
    drop(running);

    let ctx = Arc::new(ctx);
    let db = Arc::new(db);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

        loop {
            interval.tick().await;

            if let Err(e) = tick(&ctx, &db).await {
                tracing::error!("Auto-cleaner error: {}", e);
            }
        }
    });

    tracing::info!("Auto-cleaner started");
}

async fn tick(ctx: &serenity::Context, db: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cleans = db.get_all_cleans().await?;

    for clean in cleans {
        let guild = match ctx.cache.guild(serenity::GuildId::new(clean.guild_id)) {
            Some(g) => g.clone(),
            None => continue,
        };

        let channel = guild
            .channels
            .values()
            .find(|c| c.name.to_lowercase() == clean.channel_name.to_lowercase());

        let channel = match channel {
            Some(c) => c.clone(),
            None => {
                // Channel doesn't exist, remove the clean config
                db.delete_clean(clean.guild_id, &clean.channel_name).await?;
                continue;
            }
        };

        let new_remaining = clean.remaining_time - 1;

        // Send warning when time matches warning_time
        if new_remaining == clean.warning_time {
            let embed = serenity::CreateEmbed::new()
                .author(serenity::CreateEmbedAuthor::new(format!(
                    "Yuno is going to clean this channel in {} minutes. Speak now or forever hold your peace.",
                    clean.warning_time
                )));

            let _ = channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await;
        }

        // Time to clean
        if new_remaining <= 0 {
            // Delete messages
            if let Ok(messages) = channel.messages(ctx, serenity::GetMessages::new().limit(100)).await {
                let message_ids: Vec<_> = messages.iter().map(|m| m.id).collect();
                if message_ids.len() > 1 {
                    let _ = channel.delete_messages(ctx, &message_ids).await;
                }
            }

            // Send completion message
            let embed = serenity::CreateEmbed::new()
                .image("https://vignette3.wikia.nocookie.net/futurediary/images/9/94/Mirai_Nikki_-_06_-_Large_05.jpg")
                .author(serenity::CreateEmbedAuthor::new("Auto-clean: Yuno is done cleaning.").icon_url(ctx.cache.current_user().face()))
                .color(0xff51ff);

            let _ = channel.send_message(ctx, serenity::CreateMessage::new().embed(embed)).await;

            // Reset the timer
            db.set_clean(
                clean.guild_id,
                &clean.channel_name,
                clean.time_between_cleans,
                clean.warning_time,
                Some(clean.time_between_cleans * 60),
            )
            .await?;
        } else {
            // Update remaining time
            db.set_clean(
                clean.guild_id,
                &clean.channel_name,
                clean.time_between_cleans,
                clean.warning_time,
                Some(new_remaining),
            )
            .await?;
        }
    }

    Ok(())
}
