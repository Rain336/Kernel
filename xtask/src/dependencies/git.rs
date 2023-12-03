// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::Dependency;
use color_eyre::Result;
use xshell::{cmd, Shell};

pub struct GitDependency {
    pub id: &'static str,
    pub repo_url: &'static str,
    pub branch: Option<&'static str>,
    pub post_install: Option<fn(&Shell) -> Result<()>>,
}

impl Dependency for GitDependency {
    fn id(&self) -> &'static str {
        self.id
    }

    fn install(&self, sh: &Shell, _: &mut serde_json::Value) -> Result<()> {
        let mut cmd = cmd!(sh, "git clone --depth=1 --single-branch");

        if let Some(branch) = self.branch {
            cmd = cmd.arg("--branch").arg(branch);
        }

        cmd.arg(self.repo_url).arg(self.id).run()?;

        if let Some(post_install) = self.post_install {
            let dir = sh.push_dir(self.id);
            post_install(sh)?;
            drop(dir);
        }

        Ok(())
    }

    fn update(&self, sh: &Shell, _: &mut serde_json::Value) -> Result<()> {
        let _ = sh.push_dir(self.id);

        let cmd = cmd!(sh, "git pull --depth=1");

        if let Some(post_install) = self.post_install {
            let output = cmd.read()?;
            if !output.contains("Already up to date.") {
                let dir = sh.push_dir(self.id);
                post_install(sh)?;
                drop(dir);
            }
        } else {
            cmd.run()?;
        }

        Ok(())
    }
}
