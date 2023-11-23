// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Terminal UI Mode
//!
//! The terminal UI mode is currently unimplemented.
//! Only parsing it's input is supported right now.
//!
use anyhow::{bail, Result};
use std::path::PathBuf;

/// Input for the terminal UI mode.
pub struct TuiModeInput {
    pub project: Option<PathBuf>,
    pub restrict: Option<String>,
}

/// Returns an error saying the terminal UI mode isn't implemented.
pub fn run(_input: TuiModeInput) -> Result<()> {
    bail!("Terminal UI mode is not implemented yet")
}
