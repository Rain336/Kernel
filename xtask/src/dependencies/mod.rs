// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod download;
mod git;
mod hooks;
mod manager;
mod predefined;
mod rust;

pub use git::GitDependency;
pub use manager::DependencyManager;
pub use predefined::*;

use color_eyre::Result;
use std::path::PathBuf;
use xshell::{PushDir, Shell};

pub trait Dependency {
    fn id(&self) -> &'static str;
    fn install(&self, sh: &Shell, metadata: &mut serde_json::Value) -> Result<()>;
    fn update(&self, sh: &Shell, metadata: &mut serde_json::Value) -> Result<()>;
}

pub struct ResolvedDependency {
    pub path: PathBuf,
}

impl ResolvedDependency {
    pub fn change_dir(&self, sh: &Shell) {
        sh.change_dir(&self.path)
    }

    pub fn push_dir<'a>(&self, sh: &'a Shell) -> PushDir<'a> {
        sh.push_dir(&self.path)
    }
}
