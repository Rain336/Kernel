// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Theme for the Terminal Output
//!
//! The theme is compose out of 8 normal or dark colors and 8 bright colors.
//! Each being able to be set as either the foreground or background color.

/// The default foreground color used.
pub const DEFAULT_FG_COLOR: (u8, u8, u8) =
    split_color(config::value!("terminal.theme.default_foreground"));

/// The default background color used.
pub const DEFAULT_BG_COLOR: (u8, u8, u8) =
    split_color(config::value!("terminal.theme.default_background"));

/// Get a normal or dark color based on the given char.
/// The char is the last digit of it's ansi escape sequence.
pub const fn get_color(c: char) -> (u8, u8, u8) {
    match c {
        '0' => split_color(config::value!("terminal.theme.black")),
        '1' => split_color(config::value!("terminal.theme.red")),
        '2' => split_color(config::value!("terminal.theme.green")),
        '3' => split_color(config::value!("terminal.theme.yellow")),
        '4' => split_color(config::value!("terminal.theme.blue")),
        '5' => split_color(config::value!("terminal.theme.magenta")),
        '6' => split_color(config::value!("terminal.theme.cyan")),
        '7' => split_color(config::value!("terminal.theme.white")),
        _ => DEFAULT_BG_COLOR,
    }
}

/// Get a bright color based on the given char.
/// The char is the last digit of it's ansi escape sequence.
pub const fn get_bright_color(c: char) -> (u8, u8, u8) {
    match c {
        '0' => split_color(config::value!("terminal.theme.bright_black")),
        '1' => split_color(config::value!("terminal.theme.bright_red")),
        '2' => split_color(config::value!("terminal.theme.bright_green")),
        '3' => split_color(config::value!("terminal.theme.bright_yellow")),
        '4' => split_color(config::value!("terminal.theme.bright_blue")),
        '5' => split_color(config::value!("terminal.theme.bright_magenta")),
        '6' => split_color(config::value!("terminal.theme.bright_cyan")),
        '7' => split_color(config::value!("terminal.theme.bright_white")),
        _ => DEFAULT_FG_COLOR,
    }
}

const fn split_color(color: u32) -> (u8, u8, u8) {
    ((color >> 16) as u8, (color >> 8) as u8, color as u8)
}
