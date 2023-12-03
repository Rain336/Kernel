// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use std::path::PathBuf;

use crate::arguments::Bootloader;
use crate::build::BuildArguments;
use crate::utils::CommandContext;
use color_eyre::Result;

pub fn copy_kernel_binary(ctx: &mut CommandContext, build: &BuildArguments) -> Result<()> {
    let mut source: PathBuf = ctx.target_directory().into();
    source.push(build.target.as_rust_target());
    source.push(if build.release { "release" } else { "debug" });
    source.push(build.bootloader.as_bootloader_package());

    if matches!(build.bootloader, Bootloader::Rust) {
        ctx.file_system_mut().create_file("kernel-x86_64", source)?;
    } else {
        ctx.file_system_mut().create_file("system/kernel", source)?;
    }

    Ok(())
}
