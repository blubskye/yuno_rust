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
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}?mode=rwc", path))
            .await?;

        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<()> {
        // Create guilds table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS guilds (
                id TEXT PRIMARY KEY,
                prefix TEXT,
                spam_filter INTEGER DEFAULT 1,
                on_join_dm_msg TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create experiences table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS experiences (
                id INTEGER PRIMARY KEY,
                user_id TEXT,
                guild_id TEXT,
                xp INTEGER DEFAULT 0,
                level INTEGER DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create channel cleans table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS channel_cleans (
                id INTEGER PRIMARY KEY,
                guild_id TEXT,
                channel_name TEXT,
                time_between_cleans INTEGER,
                warning_time INTEGER,
                remaining_time INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create mention responses table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mention_responses (
                id INTEGER PRIMARY KEY,
                guild_id TEXT,
                trigger TEXT,
                response TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create ban images table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ban_images (
                guild_id TEXT,
                banner TEXT,
                image TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create mod actions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mod_actions (
                id INTEGER PRIMARY KEY,
                guild_id TEXT,
                moderator_id TEXT,
                target_id TEXT,
                action TEXT,
                reason TEXT,
                timestamp INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create level role mappings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS level_roles (
                id INTEGER PRIMARY KEY,
                guild_id TEXT,
                level INTEGER,
                role_id TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_experiences_user_guild ON experiences(user_id, guild_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mod_actions_guild ON mod_actions(guild_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mod_actions_moderator ON mod_actions(guild_id, moderator_id)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Guild operations
    pub async fn init_guild(&self, guild_id: u64) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO guilds(id) VALUES(?)")
            .bind(guild_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_prefix(&self, guild_id: u64) -> Result<Option<String>> {
        let result: Option<(Option<String>,)> =
            sqlx::query_as("SELECT prefix FROM guilds WHERE id = ?")
                .bind(guild_id.to_string())
                .fetch_optional(&self.pool)
                .await?;

        Ok(result.and_then(|(prefix,)| prefix))
    }

    pub async fn set_prefix(&self, guild_id: u64, prefix: &str) -> Result<()> {
        self.init_guild(guild_id).await?;
        sqlx::query("UPDATE guilds SET prefix = ? WHERE id = ?")
            .bind(prefix)
            .bind(guild_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Mod action operations
    pub async fn add_mod_action(
        &self,
        guild_id: u64,
        moderator_id: u64,
        target_id: u64,
        action: &str,
        reason: Option<&str>,
        timestamp: i64,
    ) -> Result<()> {
        self.init_guild(guild_id).await?;
        sqlx::query(
            "INSERT INTO mod_actions(guild_id, moderator_id, target_id, action, reason, timestamp) VALUES(?, ?, ?, ?, ?, ?)",
        )
        .bind(guild_id.to_string())
        .bind(moderator_id.to_string())
        .bind(target_id.to_string())
        .bind(action)
        .bind(reason)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_mod_stats(&self, guild_id: u64) -> Result<ModStats> {
        // Get action counts
        let action_counts: Vec<(String, i64)> = sqlx::query_as(
            "SELECT action, COUNT(*) as count FROM mod_actions WHERE guild_id = ? GROUP BY action",
        )
        .bind(guild_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        // Get top moderators
        let top_mods: Vec<(String, i64)> = sqlx::query_as(
            "SELECT moderator_id, COUNT(*) as count FROM mod_actions WHERE guild_id = ? GROUP BY moderator_id ORDER BY count DESC LIMIT 10",
        )
        .bind(guild_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        // Get mod counts by action
        let mod_counts: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT moderator_id, action, COUNT(*) as count FROM mod_actions WHERE guild_id = ? GROUP BY moderator_id, action",
        )
        .bind(guild_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        // Get total count
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM mod_actions WHERE guild_id = ?",
        )
        .bind(guild_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(ModStats {
            action_counts,
            top_mods,
            mod_counts,
            total: total.0,
        })
    }

    // XP operations
    pub async fn get_xp(&self, guild_id: u64, user_id: u64) -> Result<(i64, i64)> {
        let result: Option<(i64, i64)> = sqlx::query_as(
            "SELECT xp, level FROM experiences WHERE guild_id = ? AND user_id = ?",
        )
        .bind(guild_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.unwrap_or((0, 0)))
    }

    pub async fn set_xp(&self, guild_id: u64, user_id: u64, xp: i64, level: i64) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO experiences(user_id, guild_id, xp, level) VALUES(?, ?, ?, ?)
            ON CONFLICT(user_id, guild_id) DO UPDATE SET xp = ?, level = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(guild_id.to_string())
        .bind(xp)
        .bind(level)
        .bind(xp)
        .bind(level)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Auto-clean operations
    pub async fn get_clean(&self, guild_id: u64, channel_name: &str) -> Result<Option<CleanConfig>> {
        let result: Option<(i64, i64, i64)> = sqlx::query_as(
            "SELECT time_between_cleans, warning_time, remaining_time FROM channel_cleans WHERE guild_id = ? AND channel_name = ?",
        )
        .bind(guild_id.to_string())
        .bind(channel_name.to_lowercase())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(time_between, warning, remaining)| CleanConfig {
            guild_id,
            channel_name: channel_name.to_string(),
            time_between_cleans: time_between,
            warning_time: warning,
            remaining_time: remaining,
        }))
    }

    pub async fn set_clean(
        &self,
        guild_id: u64,
        channel_name: &str,
        time_between: i64,
        warning_time: i64,
        remaining_time: Option<i64>,
    ) -> Result<()> {
        let remaining = remaining_time.unwrap_or(time_between * 60);
        sqlx::query(
            r#"
            INSERT INTO channel_cleans(guild_id, channel_name, time_between_cleans, warning_time, remaining_time)
            VALUES(?, ?, ?, ?, ?)
            ON CONFLICT(guild_id, channel_name) DO UPDATE SET
                time_between_cleans = ?, warning_time = ?, remaining_time = ?
            "#,
        )
        .bind(guild_id.to_string())
        .bind(channel_name.to_lowercase())
        .bind(time_between)
        .bind(warning_time)
        .bind(remaining)
        .bind(time_between)
        .bind(warning_time)
        .bind(remaining)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_clean(&self, guild_id: u64, channel_name: &str) -> Result<()> {
        sqlx::query("DELETE FROM channel_cleans WHERE guild_id = ? AND channel_name = ?")
            .bind(guild_id.to_string())
            .bind(channel_name.to_lowercase())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_all_cleans(&self) -> Result<Vec<CleanConfig>> {
        let results: Vec<(String, String, i64, i64, i64)> = sqlx::query_as(
            "SELECT guild_id, channel_name, time_between_cleans, warning_time, remaining_time FROM channel_cleans",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(gid, cname, time_between, warning, remaining)| CleanConfig {
                guild_id: gid.parse().unwrap_or(0),
                channel_name: cname,
                time_between_cleans: time_between,
                warning_time: warning,
                remaining_time: remaining,
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct CleanConfig {
    pub guild_id: u64,
    pub channel_name: String,
    pub time_between_cleans: i64,
    pub warning_time: i64,
    pub remaining_time: i64,
}

#[derive(Debug)]
pub struct ModStats {
    pub action_counts: Vec<(String, i64)>,
    pub top_mods: Vec<(String, i64)>,
    pub mod_counts: Vec<(String, String, i64)>,
    pub total: i64,
}
