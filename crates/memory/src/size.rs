/// # Size Module
///
/// The size module contains the different valid sizes for [`crate::page::Page`] and [`crate::frame::PhysFrame`].
/// The module exports one trait, [`PageSize`] with is implemented by different valid sizes.

/// A trait implemented by zero-sized enums to represent a mappable page and frame size.
pub trait PageSize: Ord + Copy {
    /// Returns the size in bytes.
    const SIZE: u64;

    /// Returns whenever the size is supported on the current architecture.
    fn is_supported() -> bool;
}

/// A page size type for 4 KiB.
/// This size is supported by all current architectures (x86_64, AArch64, riscv64).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;

    fn is_supported() -> bool {
        true
    }
}

/// A page size type for 2 MiB.
/// This size is supported by all current architectures (x86_64, AArch64, riscv64).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size2MiB {}

impl PageSize for Size2MiB {
    const SIZE: u64 = 2097152;

    fn is_supported() -> bool {
        true
    }
}

/// A page size type for 1 GiB.
/// This size is supported by all current architectures (x86_64, AArch64, riscv64).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size1GiB {}

impl PageSize for Size1GiB {
    const SIZE: u64 = 1073741824;

    fn is_supported() -> bool {
        true
    }
}

/// A page size type for 512 GiB.
/// This size is currently only supported by the riscv64 architecture.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size512GiB {}

impl PageSize for Size512GiB {
    const SIZE: u64 = 549755813888;

    fn is_supported() -> bool {
        // Only supported on riscv64 with Sv48
        false
    }
}
