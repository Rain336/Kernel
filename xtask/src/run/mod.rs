// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
mod limine;
mod rust;

use crate::arguments::{Bootloader, Target};
use crate::build::{self, BuildArguments};
use crate::dependencies::OVMF_DEPENDENCY;
use crate::utils::CommandContext;
use clap::Args;
use color_eyre::Result;
use xshell::cmd;

/// Builds the microdragon kernel and runs it in a VM.
#[derive(Args)]
pub struct RunArguments {
    #[command(flatten)]
    build: BuildArguments,

    /// Additional QEMU arguments.
    args: Vec<String>,
}

impl RunArguments {
    pub fn run(self, mut ctx: CommandContext) -> Result<()> {
        self.build.run()?;

        println!("Collecting files...");
        self.copy_bootloader_files(&mut ctx)?;
        build::copy_kernel_binary(&mut ctx, &self.build)?;

        println!("Generating FAT image...");
        let img = ctx.target_directory().join("microdragon-run.img");
        ctx.file_system().write_fat_image(&img, *b"MICRODRAGON")?;

        println!("Starting QEMU...");
        let ovmf = ctx.resolve_dependency(&OVMF_DEPENDENCY)?;
        let (qemu, default_args, code) = match self.build.target {
            Target::X86_64 => (
                "qemu-system-x86_64",
                vec!["-cpu", "qemu64"],
                ovmf.path.join("x64/code.fd"),
            ),
            Target::AArch64 => (
                "qemu-system-aarch64",
                vec!["-M", "virt"],
                ovmf.path.join("aarch64/code.fd"),
            ),
            Target::RiscV64 => todo!(),
        };
        let extra = &self.args;

        cmd!(
            ctx.shell(),
            "{qemu} {default_args...} -drive if=pflash,format=raw,unit=0,file={code},readonly=on -drive if=ide,format=raw,file={img} -net none {extra...}"
        )
        .run()?;

        Ok(())
    }

    fn copy_bootloader_files(&self, ctx: &mut CommandContext) -> Result<()> {
        match self.build.bootloader {
            Bootloader::Limine => limine::copy_files(ctx, self.build.target),
            Bootloader::Rust => rust::copy_files(ctx),
        }
    }
}
