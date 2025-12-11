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
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Database {
    pool: Pool<Sqlite>,
    // XP batching
    pending_xp: Arc<Mutex<HashMap<String, PendingXp>>>,
}

#[derive(Debug, Clone)]
pub struct PendingXp {
    pub user_id: u64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub xp_amount: i64,
    pub added_at: i64,
}

#[derive(Debug, Clone)]
pub struct VoiceXpConfig {
    pub guild_id: u64,
    pub enabled: bool,
    pub xp_per_minute: i32,
    pub min_users: i32,
    pub ignore_afk: bool,
}

#[derive(Debug, Clone)]
pub struct ActivityLog {
    pub id: i64,
    pub guild_id: u64,
    pub user_id: u64,
    pub channel_id: u64,
    pub event_type: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct DmInbox {
    pub id: i64,
    pub user_id: u64,
    pub username: String,
    pub content: String,
    pub timestamp: i64,
    pub read_status: bool,
}

#[derive(Debug, Clone)]
pub struct BotBan {
    pub user_id: u64,
    pub banned_by: u64,
    pub reason: Option<String>,
    pub timestamp: i64,
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            pending_xp: Arc::clone(&self.pending_xp),
        }
    }
}

impl Database {
    pub async fn new(path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}?mode=rwc", path))
            .await?;

        Ok(Self {
            pool,
            pending_xp: Arc::new(Mutex::new(HashMap::new())),
        })
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

        // Voice XP config table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS voice_xp_config (
                guild_id TEXT PRIMARY KEY,
                enabled INTEGER DEFAULT 0,
                xp_per_minute INTEGER DEFAULT 5,
                min_users INTEGER DEFAULT 2,
                ignore_afk INTEGER DEFAULT 1
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Activity log table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS activity_log (
                id INTEGER PRIMARY KEY,
                guild_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                channel_id TEXT,
                event_type TEXT NOT NULL,
                old_content TEXT,
                new_content TEXT,
                timestamp INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_activity_guild ON activity_log(guild_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_activity_timestamp ON activity_log(timestamp)")
            .execute(&self.pool)
            .await?;

        // DM inbox table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS dm_inbox (
                id INTEGER PRIMARY KEY,
                user_id TEXT NOT NULL,
                username TEXT,
                content TEXT,
                timestamp INTEGER NOT NULL,
                read_status INTEGER DEFAULT 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_dm_timestamp ON dm_inbox(timestamp)")
            .execute(&self.pool)
            .await?;

        // Bot bans table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bot_bans (
                user_id TEXT PRIMARY KEY,
                banned_by TEXT,
                reason TEXT,
                timestamp INTEGER NOT NULL
            )
            "#,
        )
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

    // XP Batching operations
    pub async fn add_xp_to_batch(&self, user_id: u64, guild_id: u64, channel_id: u64, xp: i64) {
        let key = format!("{}:{}", guild_id, user_id);
        let mut pending = self.pending_xp.lock().await;

        if let Some(entry) = pending.get_mut(&key) {
            entry.xp_amount += xp;
            entry.channel_id = channel_id;
        } else {
            pending.insert(
                key,
                PendingXp {
                    user_id,
                    guild_id,
                    channel_id,
                    xp_amount: xp,
                    added_at: chrono::Utc::now().timestamp(),
                },
            );
        }
    }

    pub async fn flush_xp_batch(&self) -> Result<Vec<(u64, u64, u64, i64, i64)>> {
        let mut pending = self.pending_xp.lock().await;
        if pending.is_empty() {
            return Ok(Vec::new());
        }

        let to_process: HashMap<String, PendingXp> = std::mem::take(&mut *pending);
        drop(pending);

        let mut level_ups = Vec::new();

        for (_, xp_entry) in to_process {
            let (current_xp, current_level) = self.get_xp(xp_entry.guild_id, xp_entry.user_id).await?;
            let new_xp = current_xp + xp_entry.xp_amount;
            let new_level = ((new_xp as f64) / 100.0).sqrt() as i64;

            self.set_xp(xp_entry.guild_id, xp_entry.user_id, new_xp, new_level).await?;

            if new_level > current_level {
                level_ups.push((xp_entry.user_id, xp_entry.guild_id, xp_entry.channel_id, new_level, new_xp));
            }
        }

        Ok(level_ups)
    }

    // Voice XP config operations
    pub async fn get_voice_xp_config(&self, guild_id: u64) -> Result<VoiceXpConfig> {
        let result: Option<(i32, i32, i32, i32)> = sqlx::query_as(
            "SELECT enabled, xp_per_minute, min_users, ignore_afk FROM voice_xp_config WHERE guild_id = ?",
        )
        .bind(guild_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(result
            .map(|(enabled, xp_per_min, min_users, ignore_afk)| VoiceXpConfig {
                guild_id,
                enabled: enabled != 0,
                xp_per_minute: xp_per_min,
                min_users,
                ignore_afk: ignore_afk != 0,
            })
            .unwrap_or(VoiceXpConfig {
                guild_id,
                enabled: false,
                xp_per_minute: 5,
                min_users: 2,
                ignore_afk: true,
            }))
    }

    pub async fn set_voice_xp_config(&self, config: &VoiceXpConfig) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO voice_xp_config (guild_id, enabled, xp_per_minute, min_users, ignore_afk)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(guild_id) DO UPDATE SET
                enabled = ?, xp_per_minute = ?, min_users = ?, ignore_afk = ?
            "#,
        )
        .bind(config.guild_id.to_string())
        .bind(config.enabled as i32)
        .bind(config.xp_per_minute)
        .bind(config.min_users)
        .bind(config.ignore_afk as i32)
        .bind(config.enabled as i32)
        .bind(config.xp_per_minute)
        .bind(config.min_users)
        .bind(config.ignore_afk as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Activity log operations
    pub async fn log_activity(&self, log: &ActivityLog) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO activity_log (guild_id, user_id, channel_id, event_type, old_content, new_content, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(log.guild_id.to_string())
        .bind(log.user_id.to_string())
        .bind(log.channel_id.to_string())
        .bind(&log.event_type)
        .bind(&log.old_content)
        .bind(&log.new_content)
        .bind(log.timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_activity_logs(&self, guild_id: u64, limit: i64) -> Result<Vec<ActivityLog>> {
        let results: Vec<(i64, String, String, String, Option<String>, Option<String>, i64)> =
            sqlx::query_as(
                "SELECT id, user_id, channel_id, event_type, old_content, new_content, timestamp FROM activity_log WHERE guild_id = ? ORDER BY timestamp DESC LIMIT ?",
            )
            .bind(guild_id.to_string())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(results
            .into_iter()
            .map(|(id, uid, cid, event_type, old_content, new_content, timestamp)| ActivityLog {
                id,
                guild_id,
                user_id: uid.parse().unwrap_or(0),
                channel_id: cid.parse().unwrap_or(0),
                event_type,
                old_content,
                new_content,
                timestamp,
            })
            .collect())
    }

    // DM inbox operations
    pub async fn save_dm(&self, dm: &DmInbox) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO dm_inbox (user_id, username, content, timestamp, read_status)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(dm.user_id.to_string())
        .bind(&dm.username)
        .bind(&dm.content)
        .bind(dm.timestamp)
        .bind(dm.read_status as i32)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_dms(&self, limit: i64) -> Result<Vec<DmInbox>> {
        let results: Vec<(i64, String, Option<String>, Option<String>, i64, i32)> = sqlx::query_as(
            "SELECT id, user_id, username, content, timestamp, read_status FROM dm_inbox ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(id, uid, username, content, timestamp, read_status)| DmInbox {
                id,
                user_id: uid.parse().unwrap_or(0),
                username: username.unwrap_or_default(),
                content: content.unwrap_or_default(),
                timestamp,
                read_status: read_status != 0,
            })
            .collect())
    }

    pub async fn mark_dm_read(&self, dm_id: i64) -> Result<()> {
        sqlx::query("UPDATE dm_inbox SET read_status = 1 WHERE id = ?")
            .bind(dm_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_unread_dm_count(&self) -> Result<i64> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM dm_inbox WHERE read_status = 0")
            .fetch_one(&self.pool)
            .await?;
        Ok(result.0)
    }

    // Bot ban operations
    pub async fn add_bot_ban(&self, ban: &BotBan) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO bot_bans (user_id, banned_by, reason, timestamp)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(ban.user_id.to_string())
        .bind(ban.banned_by.to_string())
        .bind(&ban.reason)
        .bind(ban.timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_bot_ban(&self, user_id: u64) -> Result<()> {
        sqlx::query("DELETE FROM bot_bans WHERE user_id = ?")
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_bot_banned(&self, user_id: u64) -> Result<bool> {
        let result: Option<(i64,)> = sqlx::query_as("SELECT 1 FROM bot_bans WHERE user_id = ?")
            .bind(user_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        Ok(result.is_some())
    }

    pub async fn get_bot_bans(&self, limit: i64) -> Result<Vec<BotBan>> {
        let results: Vec<(String, String, Option<String>, i64)> = sqlx::query_as(
            "SELECT user_id, banned_by, reason, timestamp FROM bot_bans ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|(uid, banned_by, reason, timestamp)| BotBan {
                user_id: uid.parse().unwrap_or(0),
                banned_by: banned_by.parse().unwrap_or(0),
                reason,
                timestamp,
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
