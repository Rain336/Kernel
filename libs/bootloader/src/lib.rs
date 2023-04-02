//! # Bootlodaer Interface
//! This library implements the bootloader interface specification for the microdragon kernel.
//! The specification is written down as a markdown document and is referanced throughout this library's documentation.
//! The entry point of this library is the [`BootInfo`] struct which gets passed to the kernel's entry point [`KernelEntryPointFn`].

#![no_std]
#![deny(improper_ctypes_definitions)]
#![warn(missing_docs)]

mod memory;

pub use memory::*;

/// path to the kernel binary as suggested by the document.
pub const KERNEL_PATH: &str = "/system/kernel";

/// path to the initfs file as suggested by the document.
pub const INITFS_PATH: &str = "/system/initfs";

/// The type of the kernel's entry point function.
pub type KernelEntryPointFn = extern "C" fn(info: &mut BootInfo) -> !;

/// The revision of the bootloader interface document implemented by this library.
pub const CURRENT_REVISION: u8 = 1;

/// The [`BootInfo`] struct is the main way of how information is passed from the bootloader to the kernel.
#[repr(C)]
pub struct BootInfo {
    /// The version of this document as implememnted by the bootloader.
    /// This is to catch version mismatches between the bootloader and the kernel.
    pub revision: u8,

    /// See [`KernelPosition`].
    pub kernel_position: KernelPosition,

    /// See [`MemoryInfo`].
    pub memory_info: MemoryInfo,

    /// The physical address of the UEFI System Table, if the kernel was booted though UEFI, otherwise `0`.
    pub system_table: u64,

    /// The physical address of the ACPI Root/eXtended System Descriptor Table or `0` if unavilable.
    pub sdt_address: u64,

    /// The physical address of of where the bootloader loaded the initfs. If no initfs was supplied it should be `0`.
    pub initfs_address: u64,

    /// The length of the loaded initfs or `0` if no initfs was supplied.
    pub initfs_length: u64,
}

impl BootInfo {
    /// Checks if the structs revision matches the libraries.
    pub fn is_vaild(&self) -> bool {
        self.revision == CURRENT_REVISION
    }
}

/// The kernel position struct tells the kernel where it's stack and tls segment is located in physical memory.
#[repr(C)]
pub struct KernelPosition {
    /// The physical addresses of the start of the kernel's tls segment.
    /// The kernel should only have one tls segment.
    pub tls_address: u64,

    /// The length in bytes of the kernel's tls segment.
    pub tls_length: u64,

    /// The address of the start of the kernel's stack.
    pub stack_start: u64,

    /// The address of the end of the kernel's stack.
    pub stack_end: u64,
}

#[cfg(debug_assertions)]
extern "C" fn _assert_ffi(_: BootInfo) {}
