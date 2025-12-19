<div align="center">

# 💕 Yuno Gasai 2 (Rust Edition) 💕

### *"I'll protect this server forever... just for you~"* 💗

<img src="https://i.imgur.com/jF8Szfr.png" alt="Yuno Gasai" width="300"/>

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-pink.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-ff69b4.svg)](https://www.rust-lang.org/)
[![Serenity](https://img.shields.io/badge/Serenity-0.12-ff1493.svg)](https://github.com/serenity-rs/serenity)

*A devoted Discord bot for moderation, leveling, and anime~ ♥*

---

### 🦀 Ported to Rust... for the memes 🦀

*Because why not rewrite everything in Rust?*

---

### 💘 She loves you... and only you 💘

</div>

## 🌸 About

Yuno is a **yandere-themed Discord bot** combining powerful moderation tools with a leveling system and anime features. She'll keep your server safe from troublemakers... *because no one else is allowed near you~* 💕

This is the **Rust port** of the original JavaScript version. Why Rust? *Because we can.* 🦀

---

## 👑 Credits

*"These are the ones who gave me life~"* 💖

| Contributor | Role |
|-------------|------|
| **blubskye** | Project Owner, Rust Porter & Yuno's #1 Fan 💕🔪 |
| **Maeeen** (maeeennn@gmail.com) | Original Developer 💝 |
| **Oxdeception** | Contributor 💗 |
| **fuzzymanboobs** | Contributor 💗 |

---

## 💗 Features

<table>
<tr>
<td width="50%">

### 🔪 Moderation
*"Anyone who threatens you... I'll eliminate them~"*
- ⛔ Ban / Unban / Kick / Timeout
- 🧹 Channel cleaning & auto-clean
- 🛡️ Spam filter protection
- 👑 Mod statistics tracking
- 📊 Scan & import ban history
- 🚫 Bot-level bans (cross-guild blocking)

</td>
<td width="50%">

### ✨ Leveling System
*"Watch me make you stronger, senpai~"*
- 📊 XP & Level tracking with batching
- 🎭 Role rewards per level
- 🏆 Server leaderboards
- 🎤 Voice XP configuration
- 📝 Activity logging

</td>
</tr>
<tr>
<td width="50%">

### 🌸 Anime & Fun
*"Let me show you something cute~"*
- 🎌 Anime/manga search (Jikan API)
- 🐱 Neko images
- 🎱 8ball fortune telling
- 💬 Custom mention responses
- 📜 Yuno Gasai quotes

</td>
<td width="50%">

### ⚙️ Configuration
*"I'll be exactly what you need~"*
- 🔧 Customizable prefix
- 🎮 Slash commands + prefix commands
- 📝 Per-guild settings
- 🦀 **Blazingly fast™** (it's Rust)

</td>
</tr>
<tr>
<td width="50%">

### 🦀 Why Rust?
*"Because I'm not like other bots~"*
- 🔒 Memory safety without GC
- ⚡ Zero-cost abstractions
- 🚀 Performance that would make C jealous
- 😎 For the memes

</td>
<td width="50%">

### ⚡ Performance
*"Nothing can slow me down~"*
- 📈 Async/await with Tokio
- 💨 SQLite with sqlx
- 🧠 Efficient caching & XP batching
- 🎯 Native binary speed

</td>
</tr>
<tr>
<td width="50%">

### 📬 DM Inbox
*"I read every message you send me~"*
- 💌 Stores all DMs received
- 📖 Read/unread tracking
- 🔔 Console notifications

</td>
<td width="50%">

### 💻 Terminal Interface
*"Control me from the command line~"*
- 🖥️ Interactive admin console
- 📊 Server & status monitoring
- 🚫 Bot ban management
- 📬 DM inbox viewer

</td>
</tr>
</table>

---

## 💕 Installation

### 📋 Prerequisites

> *"Let me prepare everything for you~"* 💗

- **Rust** (install via [rustup](https://rustup.rs/))
- **SQLite3**
- **Git**

### 🌸 Setup Steps

```bash
# Clone the repository~ ♥
git clone https://github.com/blubskye/yuno_rust.git

# Enter my world~
cd yuno_rust

# Let me gather my strength... (this may take a while, Rust things~)
cargo build --release
```

### 💝 Configuration

Create a `config.json` file:

```json
{
    "discord_token": "YOUR_DISCORD_BOT_TOKEN",
    "default_prefix": ".",
    "database_path": "yuno.db",
    "master_users": ["YOUR_USER_ID"],
    "spam_max_warnings": 3
}
```

Or just set the `DISCORD_TOKEN` environment variable if you're lazy~

### 🚀 Running

```bash
# Release mode (recommended)
cargo run --release

# Or run the built binary directly
./target/release/yuno_gasai
```

---

## 💖 Commands Preview

| Command | Description |
|---------|-------------|
| `/ping` | *"I'm always here for you~"* 💓 |
| `/ban` | *"They won't bother you anymore..."* 🔪 |
| `/kick` | *"Get out!"* 👢 |
| `/timeout` | *"Think about what you did..."* ⏰ |
| `/clean` | *"Let me tidy up~"* 🧹 |
| `/mod-stats` | *"Look at all we've done together~"* 📊 |
| `/xp` | *"Look how strong you've become!"* ✨ |
| `/8ball` | *"Let fate decide~"* 🎱 |
| `/delay` | *"Just a bit longer..."* ⏳ |
| `/source` | *"See how I was made~"* 📜 |
| `/anime` | *"Let me find that anime for you~"* 🎌 |
| `/manga` | *"Reading is romantic~"* 📖 |
| `/neko` | *"So cute!"* 🐱 |
| `/quote` | *"Words from my heart~"* 💕 |
| `/leaderboard` | *"See who loves you most~"* 🏆 |
| `/set-level` | *"I'll make you stronger~"* ⬆️ |
| `/config` | *"View our settings~"* ⚙️ |
| `/exportbans` | *"Keep a record~"* 📤 |
| `/importbans` | *"Restore order~"* 📥 |

*Use `/help` to see all available commands!*

---

## 💻 Terminal Commands

*"I'll listen to your every command~"* 🖥️

When running, Yuno provides an interactive terminal interface:

| Command | Description |
|---------|-------------|
| `help` | Show available terminal commands |
| `servers` | List all connected Discord servers |
| `inbox [count]` | View DM inbox (marks as read) |
| `botban <user_id> [reason]` | Ban a user from using the bot |
| `botunban <user_id>` | Remove a bot-level ban |
| `botbanlist` | List all bot-banned users |
| `status` | Show bot connection status |
| `quit` / `exit` | Shutdown the bot gracefully |

---

## 📜 License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)** 💕

### 💘 What This Means For You~

*"I want to share everything with you... and everyone else too~"* 💗

The AGPL-3.0 is a **copyleft license** that ensures this software remains free and open. Here's what you need to know:

#### ✅ You CAN:
- 💕 **Use** this bot for any purpose (personal, commercial, whatever~)
- 🔧 **Modify** the code to your heart's content
- 📤 **Distribute** copies to others
- 🌐 **Run** it as a network service (like a public Discord bot)

#### 📋 You MUST:
- 📖 **Keep it open source** - If you modify and distribute this code, your version must also be AGPL-3.0
- 🔗 **Provide source access** - Users of your modified bot must be able to get the source code
- 📝 **State changes** - Document what you've modified from the original
- 💌 **Include license** - Keep the LICENSE file and copyright notices intact

#### 🌐 The Network Clause (This is the important part!):
*"Even if we're apart... I'll always be connected to you~"* 💗

Unlike regular GPL, **AGPL has a network provision**. This means:
- If you run a **modified version** of this bot as a public service (like hosting it for others to use on Discord)
- You **MUST** make your complete source code available to users
- The `/source` command in this bot helps satisfy this requirement!

#### ❌ You CANNOT:
- 🚫 Make it closed source
- 🚫 Remove the license or copyright notices
- 🚫 Use a different license for modified versions
- 🚫 Hide your modifications if you run it as a public service

#### 💡 In Simple Terms:
> *"If you use my code to create something, you must share it with everyone too~ That's only fair, right?"* 💕

This ensures that improvements to the bot benefit the entire community, not just one person. Yuno wants everyone to be happy~ 💗

See the [LICENSE](LICENSE) file for the full legal text.

**Source Code:** https://github.com/blubskye/yuno_rust

---

## 🔗 Source Code

*"I have nothing to hide from you~"* 💕

This bot is **open source** under AGPL-3.0:
- **🦀 Rust version**: https://github.com/blubskye/yuno_rust
- **📦 Original JS version**: https://github.com/japaneseenrichmentorganization/Yuno-Gasai-2

---

<div align="center">

### 💘 *"You'll stay with me forever... right?"* 💘

**Made with obsessive love** 💗 **and rewritten in Rust for the memes** 🦀

*Yuno will always be watching over your server~* 👁️💕

---

⭐ *Star this repo if Yuno has captured your heart~* ⭐

</div>
