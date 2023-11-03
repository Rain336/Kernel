//! # Parsing of escape sequences for Terminal Output.
//!
//! The parsing is done though a state machine enum [`EscapeMode`].
//! On start, the state machine is [`EscapeMode::None`] state.
//! When a `\x1B` char is encountered, the state changes to [`EscapeMode::ExpectOpenBrace`] and following characters aren't printed.
//!
//! In [`EscapeMode::ExpectOpenBrace`] state, a `[` needs to follow, or the state is returned to [`EscapeMode::None`] and the invalid char printed.
//! If a `[` was supplied, the state changes to [`EscapeMode::StartSequence`].
//!
//! From [`EscapeMode::StartSequence`], the state machine branches into:
//! - If the incoming char is `3`, go into [`EscapeMode::ForegroundColor`] state.
//! - If the incoming char is `4`, go into [`EscapeMode::BackgroundColor`] state.
//! - If the incoming char is `9`, go into [`EscapeMode::ForegroundBrightColor`] state.
//! - If the incoming char is `1`, go into [`EscapeMode::ExpectZero`] state.
//!
//! In [`EscapeMode::ForegroundColor`], the next char is passed to [`theme::get_color`] to set the foreground color and
//! sate is changed to [`EscapeMode::FinishSequence`].
//!
//! In [`EscapeMode::BackgroundColor`], the next char is passed to [`theme::get_color`] to set the background color and
//! sate is changed to [`EscapeMode::FinishSequence`].
//!
//! In [`EscapeMode::ForegroundBrightColor`], the next char is passed to [`theme::get_bright_color`] to set the foreground color and
//! sate is changed to [`EscapeMode::FinishSequence`].
//!
//! In [`EscapeMode::ExpectZero`], the next char has to be `0` or state is changed back to [`EscapeMode::None`].
//! If a `0` was encountered, state is changed to [`EscapeMode::BackgroundBrightColor`].
//!
//! In [`EscapeMode::BackgroundBrightColor`], the next char is passed to [`theme::get_bright_color`] to set the background color and
//! sate is changed to [`EscapeMode::FinishSequence`].
//!
//! In [`EscapeMode::FinishSequence`], if an `m` is encountered, the state is changed into [`EscapeMode::None`].
//! If a `;` is encountered, state is changed to [`EscapeMode::StartSequence`].
use crate::theme;

/// State Machine for processing escape sequences.
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum EscapeMode {
    None,
    ExpectOpenBrace,
    StartSequence,
    ForegroundColor,
    BackgroundColor,
    ForegroundBrightColor,
    BackgroundBrightColor,
    ExpectZero,
    FinishSequence,
}

impl EscapeMode {
    /// Runs the state machine. See module doc for how it works.
    pub fn process(&mut self, fg: &mut (u8, u8, u8), bg: &mut (u8, u8, u8), c: char) -> bool {
        match self {
            EscapeMode::ExpectOpenBrace if c == '[' => {
                *self = EscapeMode::StartSequence;
            }

            EscapeMode::StartSequence if c == '3' => {
                *self = EscapeMode::ForegroundColor;
            }
            EscapeMode::StartSequence if c == '4' => {
                *self = EscapeMode::BackgroundColor;
            }
            EscapeMode::StartSequence if c == '9' => {
                *self = EscapeMode::ForegroundBrightColor;
            }
            EscapeMode::StartSequence if c == '1' => *self = EscapeMode::ExpectZero,

            EscapeMode::ExpectZero if c == '0' => {
                *self = EscapeMode::BackgroundBrightColor;
            }

            EscapeMode::ForegroundColor => {
                if c == '9' {
                    *fg = theme::DEFAULT_FG_COLOR;
                    *bg = theme::DEFAULT_BG_COLOR;
                } else {
                    *fg = theme::get_color(c);
                }
                *self = EscapeMode::FinishSequence;
            }

            EscapeMode::BackgroundColor => {
                *bg = theme::get_color(c);
                *self = EscapeMode::FinishSequence;
            }

            EscapeMode::ForegroundBrightColor => {
                *fg = theme::get_bright_color(c);
                *self = EscapeMode::FinishSequence;
            }

            EscapeMode::BackgroundBrightColor => {
                *bg = theme::get_bright_color(c);
                *self = EscapeMode::FinishSequence;
            }

            EscapeMode::FinishSequence if c == ';' => {
                *self = EscapeMode::StartSequence;
            }
            EscapeMode::FinishSequence if c == 'm' => {
                *self = EscapeMode::None;
            }

            _ => {
                *self = EscapeMode::None;
                return false;
            }
        }

        true
    }
}
