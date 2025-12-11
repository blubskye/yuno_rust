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

use crate::database::{BotBan, Database};
use poise::serenity_prelude as serenity;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref TERMINAL_RUNNING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

pub async fn start(ctx: serenity::Context, db: Database) {
    let mut running = TERMINAL_RUNNING.lock().await;
    if *running {
        return;
    }
    *running = true;
    drop(running);

    let ctx = Arc::new(ctx);
    let db = Arc::new(db);

    tokio::spawn(async move {
        println!("\n\x1b[38;2;200;140;141m Terminal ready! Type 'help' for commands~\x1b[0m\n");

        let stdin = io::stdin();
        loop {
            print!("\x1b[38;2;200;140;141myuno>\x1b[0m ");
            let _ = io::stdout().flush();

            let mut input = String::new();
            if stdin.lock().read_line(&mut input).is_err() {
                continue;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            let parts: Vec<&str> = input.split_whitespace().collect();
            let command = parts[0].to_lowercase();
            let args: Vec<&str> = parts.iter().skip(1).cloned().collect();

            if let Err(e) = process_command(&ctx, &db, &command, &args).await {
                eprintln!("Error: {}", e);
            }
        }
    });

    tracing::info!("Terminal started");
}

async fn process_command(
    ctx: &serenity::Context,
    db: &Database,
    command: &str,
    args: &[&str],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match command {
        "help" => show_help(),
        "servers" => show_servers(ctx).await,
        "inbox" => handle_inbox(db, args).await?,
        "botban" => handle_bot_ban(db, args).await?,
        "botunban" => handle_bot_unban(db, args).await?,
        "botbanlist" => show_bot_ban_list(db).await?,
        "status" => show_status(ctx, db).await?,
        "quit" | "exit" => {
            println!("Shutting down...");
            std::process::exit(0);
        }
        _ => println!("Unknown command: {}. Type 'help' for available commands.", command),
    }
    Ok(())
}

fn show_help() {
    println!(
        r#"
Available Commands:
  help        - Show this help message
  servers     - List connected servers
  inbox       - View DM inbox (inbox [count])
  botban      - Ban user from bot (botban <user_id> [reason])
  botunban    - Unban user from bot (botunban <user_id>)
  botbanlist  - List all bot-banned users
  status      - Show bot status
  quit/exit   - Shutdown the bot
"#
    );
}

async fn show_servers(ctx: &serenity::Context) {
    let guilds = ctx.cache.guilds();
    println!("\nConnected to {} server(s):\n", guilds.len());

    for guild_id in guilds {
        if let Some(guild) = ctx.cache.guild(guild_id) {
            println!(
                "  {} (ID: {}) - {} members",
                guild.name, guild.id, guild.member_count
            );
        }
    }
    println!();
}

async fn handle_inbox(db: &Database, args: &[&str]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let limit: i64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(10);

    let dms = db.get_dms(limit).await?;
    let unread = db.get_unread_dm_count().await?;

    println!("\nDM Inbox ({} unread):\n", unread);

    if dms.is_empty() {
        println!("  No messages in inbox.");
    } else {
        for dm in &dms {
            let status = if dm.read_status { " " } else { "*" };
            let time = chrono::DateTime::from_timestamp(dm.timestamp, 0)
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let preview = if dm.content.len() > 50 {
                format!("{}...", &dm.content[..50])
            } else {
                dm.content.clone()
            };
            println!(
                "  {}[{}] {} - {} ({}): {}",
                status, dm.id, time, dm.username, dm.user_id, preview
            );

            // Mark as read
            if !dm.read_status {
                db.mark_dm_read(dm.id).await?;
            }
        }
    }
    println!();
    Ok(())
}

async fn handle_bot_ban(db: &Database, args: &[&str]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.is_empty() {
        println!("Usage: botban <user_id> [reason]");
        return Ok(());
    }

    let user_id: u64 = match args[0].parse() {
        Ok(id) => id,
        Err(_) => {
            println!("Invalid user ID.");
            return Ok(());
        }
    };

    let reason = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        Some("Banned via terminal".to_string())
    };

    db.add_bot_ban(&BotBan {
        user_id,
        banned_by: 0, // Terminal
        reason,
        timestamp: chrono::Utc::now().timestamp(),
    })
    .await?;

    println!("User {} has been banned from the bot.", user_id);
    Ok(())
}

async fn handle_bot_unban(db: &Database, args: &[&str]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if args.is_empty() {
        println!("Usage: botunban <user_id>");
        return Ok(());
    }

    let user_id: u64 = match args[0].parse() {
        Ok(id) => id,
        Err(_) => {
            println!("Invalid user ID.");
            return Ok(());
        }
    };

    if !db.is_bot_banned(user_id).await? {
        println!("User {} is not bot-banned.", user_id);
        return Ok(());
    }

    db.remove_bot_ban(user_id).await?;
    println!("User {} has been unbanned from the bot.", user_id);
    Ok(())
}

async fn show_bot_ban_list(db: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bans = db.get_bot_bans(50).await?;

    println!("\nBot-banned users ({}):\n", bans.len());

    if bans.is_empty() {
        println!("  No users are bot-banned.");
    } else {
        for ban in bans {
            let time = chrono::DateTime::from_timestamp(ban.timestamp, 0)
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            println!(
                "  {} - {} (banned {})",
                ban.user_id,
                ban.reason.unwrap_or_else(|| "No reason".to_string()),
                time
            );
        }
    }
    println!();
    Ok(())
}

async fn show_status(
    ctx: &serenity::Context,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let guilds = ctx.cache.guilds();
    let total_members: u64 = guilds
        .iter()
        .filter_map(|gid| ctx.cache.guild(*gid))
        .map(|g| g.member_count)
        .sum();
    let unread_dms = db.get_unread_dm_count().await?;
    let bot_bans = db.get_bot_bans(1000).await?.len();
    // Latency not directly available in this serenity version

    println!(
        r#"
Bot Status:
  Connection: Connected
  Servers: {}
  Total Members: {}
  Unread DMs: {}
  Bot Bans: {}
"#,
        guilds.len(),
        total_members,
        unread_dms,
        bot_bans,
    );

    Ok(())
}
