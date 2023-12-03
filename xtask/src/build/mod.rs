// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod kernel;

pub use kernel::copy_kernel_binary;

use crate::arguments::{Bootloader, Target};
use clap::Args;
use color_eyre::eyre::anyhow;
use color_eyre::Result;
use xshell::{cmd, Shell};

/// Builds the microdragon kernel.
#[derive(Args)]
pub struct BuildArguments {
    /// Specifies the target CPU architecture to build for.
    #[arg(short, long, value_enum, default_value_t)]
    pub target: Target,

    /// Specifies the bootloader to build for.
    #[arg(short, long, value_enum, default_value_t)]
    pub bootloader: Bootloader,

    /// Specifies if a release build should be done.
    #[arg(short, long, default_value_t)]
    pub release: bool,
}

impl BuildArguments {
    pub fn run(&self) -> Result<()> {
        if !self.bootloader.supports_target(self.target) {
            return Err(anyhow!(
                "The selected bootloader ({}) does not support the selected target ({})",
                self.bootloader,
                self.target
            ));
        }

        let sh = Shell::new()?;

        install_target_if_needed(&sh, self.target)?;

        let target = self.target.as_rust_target();
        let bootloader = self.bootloader.as_bootloader_package();
        let release = if self.release {
            Some("--release")
        } else {
            None
        };

        cmd!(
            sh,
            "cargo build --target {target} --package {bootloader} {release...}"
        )
        .run()?;

        Ok(())
    }
}

fn install_target_if_needed(sh: &Shell, target: Target) -> Result<()> {
    let installed = cmd!(sh, "rustup target list --installed").read()?;
    let targets: Vec<&str> = installed.lines().collect();
    let target = target.as_rust_target();

    if !targets.contains(&target) {
        println!("Rust target {} not installed. Installing...", target);
        cmd!(sh, "rustup target add {target}").run()?;
    }

    Ok(())
}
