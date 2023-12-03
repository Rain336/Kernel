// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::dependencies::Dependency;
use color_eyre::eyre::anyhow;
use color_eyre::Result;
use serde_json::Value;
use std::fs;
use xshell::{cmd, Shell};

pub struct RustBootloaderDependency {
    pub version: &'static str,
}

impl Dependency for RustBootloaderDependency {
    fn id(&self) -> &'static str {
        "RustBoot"
    }

    fn install(&self, sh: &Shell, metadata: &mut Value) -> Result<()> {
        let target = sh.current_dir().join(self.id());
        if target.exists() {
            fs::remove_dir_all(&target)?;
        }

        let version = self.version;
        cmd!(
            sh,
            "cargo install bootloader-x86_64-uefi --version {version} --locked --target x86_64-unknown-uefi -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem --root {target}"
        )
        .run()?;

        *metadata = Value::String(self.version.to_string());

        Ok(())
    }

    fn update(&self, sh: &Shell, metadata: &mut Value) -> Result<()> {
        let Value::String(version) = metadata else {
            return Err(anyhow!("Expected metadata to be a string"));
        };

        if version == self.version {
            return Ok(());
        }

        self.install(sh, metadata)
    }
}
