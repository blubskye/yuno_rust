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

mod moderation;
mod utility;
mod fun;

use crate::{Data, Error};

pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        // Moderation commands
        moderation::ban(),
        moderation::kick(),
        moderation::unban(),
        moderation::timeout(),
        moderation::clean(),
        moderation::mod_stats(),
        moderation::scan_bans(),

        // Utility commands
        utility::ping(),
        utility::help(),
        utility::prefix(),
        utility::delay(),
        utility::auto_clean(),
        utility::xp(),
        utility::source(),

        // Fun commands
        fun::eight_ball(),
    ]
}
