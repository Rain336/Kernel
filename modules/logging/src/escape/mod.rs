// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod mode;

pub use mode::EscapeMode;

use crate::theme;

pub struct EscapeSequence {
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
    escape_mode: EscapeMode,
}

impl EscapeSequence {
    pub fn new() -> Self {
        EscapeSequence {
            fg: theme::DEFAULT_FG_COLOR,
            bg: theme::DEFAULT_BG_COLOR,
            escape_mode: EscapeMode::None,
        }
    }

    /// Gets a color with the given intensity between the current background and foreground color.
    /// It tries to estimate a gradient between the background and foreground color with `intensity` being the percentage.
    pub const fn apply_intensity(&self, intensity: u8) -> (u8, u8, u8) {
        let inv = 255 - intensity;
        let red =
            ((self.fg.0 as u16 * intensity as u16) / 256) + ((self.bg.0 as u16 * inv as u16) / 256);
        let green =
            ((self.fg.1 as u16 * intensity as u16) / 256) + ((self.bg.1 as u16 * inv as u16) / 256);
        let blue =
            ((self.fg.2 as u16 * intensity as u16) / 256) + ((self.bg.2 as u16 * inv as u16) / 256);

        (red as u8, green as u8, blue as u8)
    }

    /// Tries to run the process command on the escape mode, if it isn't [`EscapeMode::None`].
    pub fn try_process(&mut self, c: char) -> bool {
        self.escape_mode != EscapeMode::None
            && self.escape_mode.process(&mut self.fg, &mut self.bg, c)
    }

    /// Sets the escape mode to [`EscapeMode::ExpectOpenBrace`] to start a new escape sequence.
    pub fn start(&mut self) {
        self.escape_mode = EscapeMode::ExpectOpenBrace;
    }
}
