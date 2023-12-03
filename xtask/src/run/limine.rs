// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::arguments::Target;
use crate::dependencies::LIMINE_DEPENDENCY;
use crate::utils::CommandContext;
use color_eyre::Result;

pub fn copy_files(ctx: &mut CommandContext, target: Target) -> Result<()> {
    let dep = ctx.resolve_dependency(&LIMINE_DEPENDENCY)?;
    let cfg = ctx
        .workspace_directory()
        .join("bootloader/limine/limine.cfg");
    let fs = ctx.file_system_mut();

    fs.create_file("limine/limine.cfg", cfg)?;

    match target {
        Target::X86_64 => fs.create_file("EFI/BOOT/BOOTX64.EFI", dep.path.join("BOOTX64.EFI")),
        Target::AArch64 => fs.create_file("EFI/BOOT/BOOTAA64.EFI", dep.path.join("BOOTAA64.EFI")),
        Target::RiscV64 => {
            fs.create_file("EFI/BOOT/BOOTRISCV64.EFI", dep.path.join("BOOTRISCV64.EFI"))
        }
    }
}
