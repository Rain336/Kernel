// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::Dependency;
use color_eyre::eyre::{anyhow, Result};
use serde_json::Value;
use std::fs::File;
use std::io;
use std::path::Path;
use xshell::Shell;

pub struct DownloadDependency {
    pub id: &'static str,
    pub url: &'static str,
    pub file_name: &'static str,
    pub post_install: Option<fn(&Path, &Shell) -> Result<()>>,
}

impl Dependency for DownloadDependency {
    fn id(&self) -> &'static str {
        self.id
    }

    fn install(&self, sh: &Shell, metadata: &mut Value) -> Result<()> {
        let mut reader = ureq::get(self.url).call()?.into_reader();
        let path = sh.current_dir().join(self.file_name);
        let mut file = File::create(&path)?;
        io::copy(&mut reader, &mut file)?;

        if let Some(post_install) = self.post_install {
            post_install(&path, sh)?;
        }

        *metadata = Value::String(self.url.to_string());

        Ok(())
    }

    fn update(&self, sh: &Shell, metadata: &mut Value) -> Result<()> {
        let Value::String(url) = metadata else {
            return Err(anyhow!("Expected metadata to be a string"));
        };

        if url == self.url {
            Ok(())
        } else {
            self.install(sh, metadata)
        }
    }
}
