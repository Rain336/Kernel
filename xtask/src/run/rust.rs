// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::dependencies::RUST_BOOTLOADER;
use crate::utils::CommandContext;
use color_eyre::Result;

pub fn copy_files(ctx: &mut CommandContext) -> Result<()> {
    let dep = ctx.resolve_dependency(&RUST_BOOTLOADER)?;
    let efi = dep.path.join("bin/bootloader-x86_64-uefi.efi");

    let boot_json = ctx.workspace_directory().join("bootloader/rust/boot.json");
    let fs = ctx.file_system_mut();

    fs.create_file("boot.json", boot_json)?;
    fs.create_file("EFI/BOOT/BOOTX64.EFI", efi)
}
