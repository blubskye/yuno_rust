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

mod commands;
mod config;
mod database;
mod modules;
mod utils;

use anyhow::Result;
use poise::serenity_prelude as serenity;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// User data passed to all command invocations
pub struct Data {
    pub db: database::Database,
    pub config: config::BotConfig,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// ASCII art banner
const BANNER: &str = r#"
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣯⢻⡘⢷⠀⠀⠀⠀⠀⠀⣿⣦⠹⡄⠀⢿⠟⣿⡆⠀⠸⡄⠀⢸⡄⠀⠀⠀⠀⠀⠀⠀⠀⣧
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⡄⠀⢸⣤⣳⡘⣇⠀⠀⠀⠀⠀⢸⣟⣆⢻⣆⢸⡆⢹⣿⣄⠀⣷⠀⢰⡇⠀⠀⠀⠀⠀⠀⠀⠀⢸
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠸⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣇⠀⠈⣆⠹⣿⣸⡇⡄⠀⠀⠀⢸⢧⠀⠈⠻⣆⢿⠀⠉⢻⡆⢹⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⢸
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⣷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⣟⣦⠀⠘⣆⠘⢷⣷⠹⣆⣀⣀⣸⣿⣧⣀⣀⣈⡳⡄⢸⠀⢹⡀⡀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⢸

                    ♥ Yuno Gasai (Rust) ♥
           "I'll protect this server forever... just for you~"
"#;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Print banner
    println!("\x1b[38;2;200;140;141m{}\x1b[0m", BANNER);

    // Load configuration
    info!("Loading configuration...");
    let bot_config = config::BotConfig::load()?;
    info!("Configuration loaded.");

    // Initialize database
    info!("Initializing database...");
    let db = database::Database::new(&bot_config.database_path).await?;
    db.init().await?;
    info!("Database initialized.");

    let token = bot_config.discord_token.clone();
    let default_prefix = bot_config.default_prefix.clone();

    // Set up the framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::get_commands(),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(default_prefix),
                dynamic_prefix: Some(|ctx| {
                    Box::pin(async move {
                        let data = ctx.data();
                        if let Some(guild_id) = ctx.guild_id {
                            if let Ok(Some(prefix)) = data.db.get_prefix(guild_id.get()).await {
                                return Ok(Some(prefix));
                            }
                        }
                        Ok(Some(data.config.default_prefix.clone()))
                    })
                }),
                ..Default::default()
            },
            event_handler: |ctx, event, framework, data| {
                Box::pin(modules::event_handler(ctx, event, framework, data))
            },
            on_error: |error| {
                Box::pin(async move {
                    if let Err(e) = poise::builtins::on_error(error).await {
                        tracing::error!("Error handling error: {}", e);
                    }
                })
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                info!("Registering slash commands...");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                info!("Slash commands registered.");

                Ok(Data { db, config: bot_config })
            })
        })
        .build();

    // Create the client
    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::GUILD_BANS;

    info!("Connecting to Discord...");
    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
