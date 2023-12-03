// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::build::BuildArguments;
use crate::iso::IsoArguments;
use crate::run::RunArguments;
use clap::{Parser, ValueEnum};
use std::fmt::{self, Display, Formatter};

/// Task runner for the microdragon kernel.
#[derive(Parser)]
pub enum ProgramArguments {
    Build(BuildArguments),
    Run(RunArguments),
    Iso(IsoArguments),

    /// Updates the license header in rust files.
    License,
}

#[derive(ValueEnum, Default, Clone, Copy)]
pub enum Target {
    /// Specifies the 64-bit Intel / AMD Architecture.
    #[default]
    #[value(name = "x86_64")]
    X86_64,

    /// Specifies the 64-bit Arm Architecture.
    #[value(name = "aarch64")]
    AArch64,

    /// Specifies the 64-bit Risc-V Architecture.
    #[value(name = "riscv64")]
    RiscV64,
}

impl Target {
    pub fn as_rust_target(self) -> &'static str {
        match self {
            Target::X86_64 => "x86_64-unknown-none",
            Target::AArch64 => "aarch64-unknown-none-softfloat",
            Target::RiscV64 => "riscv64imac-unknown-none-elf",
        }
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Target::X86_64 => f.write_str("x86_64"),
            Target::AArch64 => f.write_str("aarch64"),
            Target::RiscV64 => f.write_str("riscv64"),
        }
    }
}

#[derive(ValueEnum, Default, Clone, Copy)]
pub enum Bootloader {
    /// Specifies the Limine Bootloader.
    #[default]
    Limine,

    /// Specifies the Rust Bootloader.
    Rust,
}

impl Bootloader {
    pub fn as_bootloader_package(self) -> &'static str {
        match self {
            Bootloader::Limine => "microdragon-limine",
            Bootloader::Rust => "microdragon-rust",
        }
    }

    pub fn supports_target(self, target: Target) -> bool {
        match self {
            Bootloader::Limine => true,
            Bootloader::Rust => matches!(target, Target::X86_64),
        }
    }
}

impl Display for Bootloader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Bootloader::Limine => f.write_str("Limine"),
            Bootloader::Rust => f.write_str("Rust Bootloader"),
        }
    }
}

#[derive(ValueEnum, Default, Clone, Copy)]
pub enum Firmware {
    /// Specifies the Bios firmware.
    #[default]
    Bios,

    /// Specifies the Uefi Firmware.
    Uefi,
}

impl Display for Firmware {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Firmware::Bios => f.write_str("Bios"),
            Firmware::Uefi => f.write_str("Uefi"),
        }
    }
}
