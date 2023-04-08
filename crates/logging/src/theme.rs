//! # Theme for the Terminal Output
//!
//! The theme is compose out of 8 normal or dark colors and 8 birght colors.
//! Each being able to be set as either the foreground or background color.

/// The default foreground color used.
pub const DEFAULT_FG_COLOR: (u8, u8, u8) = (204, 204, 204);

/// The default background color used.
pub const DEFAULT_BG_COLOR: (u8, u8, u8) = (12, 12, 12);

/// Get a normal or drak color based on the given char.
/// The char is the last digit of it's ansi escape sequence.
pub const fn get_color(c: char) -> (u8, u8, u8) {
    match c {
        '0' => (12, 12, 12),
        '1' => (197, 15, 31),
        '2' => (19, 161, 14),
        '3' => (193, 156, 0),
        '4' => (0, 55, 218),
        '5' => (136, 23, 152),
        '6' => (58, 150, 221),
        '7' => (204, 204, 204),
        _ => DEFAULT_BG_COLOR,
    }
}

/// Get a bright color based on the given char.
/// The char is the last digit of it's ansi escape sequence.
pub const fn get_bright_color(c: char) -> (u8, u8, u8) {
    match c {
        '0' => (118, 118, 118),
        '1' => (231, 72, 86),
        '2' => (22, 198, 12),
        '3' => (249, 241, 165),
        '4' => (59, 120, 255),
        '5' => (180, 0, 158),
        '6' => (97, 214, 214),
        '7' => (242, 242, 242),
        _ => DEFAULT_FG_COLOR,
    }
}
