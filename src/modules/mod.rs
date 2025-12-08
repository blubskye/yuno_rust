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

use crate::{Data, Error};
use poise::serenity_prelude as serenity;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_user } => {
            tracing::info!("Connected as {}", data_user.name);

            // Start auto-cleaner task
            auto_cleaner::start(ctx.clone(), data.db.clone()).await;
        }

        serenity::FullEvent::Message { new_message } => {
            // Skip bot messages
            if new_message.author.bot {
                return Ok(());
            }

            // Run spam filter
            if let Some(guild_id) = new_message.guild_id {
                spam_filter::process_message(ctx, new_message, guild_id, data).await?;
            }
        }

        serenity::FullEvent::GuildBanAddition { guild_id, banned_user } => {
            tracing::info!(
                "User {} was banned from guild {}",
                banned_user.name,
                guild_id
            );
        }

        _ => {}
    }

    Ok(())
}
