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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

const DEFAULT_CONFIG_PATH: &str = "config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    #[serde(default = "default_prefix")]
    pub default_prefix: String,

    pub discord_token: String,

    #[serde(default = "default_database_path")]
    pub database_path: String,

    #[serde(default)]
    pub master_users: Vec<String>,

    #[serde(default = "default_spam_max_warnings")]
    pub spam_max_warnings: u32,

    #[serde(default)]
    pub ban_default_image: Option<String>,

    #[serde(default)]
    pub dm_message: Option<String>,

    #[serde(default)]
    pub insufficient_permissions_message: Option<String>,
}

fn default_prefix() -> String {
    ".".to_string()
}

fn default_database_path() -> String {
    "yuno.db".to_string()
}

fn default_spam_max_warnings() -> u32 {
    3
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            default_prefix: default_prefix(),
            discord_token: String::new(),
            database_path: default_database_path(),
            master_users: Vec::new(),
            spam_max_warnings: default_spam_max_warnings(),
            ban_default_image: None,
            dm_message: None,
            insufficient_permissions_message: None,
        }
    }
}

impl BotConfig {
    pub fn load() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());

        if Path::new(&config_path).exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: BotConfig = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            // Create default config file
            let config = BotConfig::default();
            let contents = serde_json::to_string_pretty(&config)?;
            std::fs::write(&config_path, contents)?;
            anyhow::bail!(
                "Config file created at {}. Please fill in your Discord token.",
                config_path
            );
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, contents)?;
        Ok(())
    }

    pub fn is_master_user(&self, user_id: &str) -> bool {
        self.master_users.contains(&user_id.to_string())
    }
}
