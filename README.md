# Microdragon

A microkernel written in Rust, trying to bridge the gap between embedded and general operating systems

I am currently focusing on x86_64, but definitely want to support AArch64 and riscv too.

## Component Overview

`libs/hpet`
Library crate for controlling the High Precision Event Timer (HPET) found in most x86_64 platforms. The library is finished and can be used by other kernels too in theory.

`libs/bootloader`
Implements struct and type definitions to bootstrap microdragon.
I plan on switching from the custom bootloader protocol defined in this crate to [Limine](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md) in the near future.

`crates/acpi`
Implements the interaction between the kernel and the Advanced Configuration and Power Interface (ACPI) firmware (The ACPI driver). I try to use as little from ACPI as possible, since most of ACPI should be handled by a userspace service.

`crates/bootloader_uefi`
UEFI bootloader for microdragon.
This implements the bootloader protocol from `libs/bootloader`.
I plan on switching to [Limine](https://github.com/limine-bootloader/limine/blob/trunk/PROTOCOL.md) in the near future, so this crate will become redundant.

`crates/common`
Common types used by different crates.
Currently this only contains some constructs for synchronisation between processors.

`crates/memory`
A Memory Management library for x86_64, AArch64 and riscv inspired by the `x86_64` crate.

`crates/platform-x86_64`
The kernel for x86_64 system.
This crate contains all x86_64 specific code and is getting build when targeting x86_64.

`docs`
An mdBook describing the kernel and it's component.
It's not quite up to date, but I try my best to keep it relevant for people.
