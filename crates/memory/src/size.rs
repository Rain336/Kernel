/// # Size Module
///
/// The size module contains the diffrent vaild sizes for [`super::page::Page`] and [`super::frame::PhysFrame`].
/// The module exports one trait, [`PageSize`] with is implemented by different vaild sizes.

/// A trait implemented by zero-sized enums to represent a mappable page and frame size.
pub trait PageSize: Ord + Copy {
    /// Returns the size in bytes.
    const SIZE: u64;

    /// Rewturns whenever the size is supported on the current archetecture.
    fn is_supported() -> bool;
}

/// A page size type for 4 kibibytes.
/// This size is supported by all current achetectures (x86_64, AArch64, riscv64).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size4KiB {}

impl PageSize for Size4KiB {
    const SIZE: u64 = 4096;

    fn is_supported() -> bool {
        true
    }
}

/// A page size type for 2 mebibytes.
/// This size is supported by all current achetectures (x86_64, AArch64, riscv64).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size2MiB {}

impl PageSize for Size2MiB {
    const SIZE: u64 = 2097152;

    fn is_supported() -> bool {
        true
    }
}

/// A page size type for 1 gibibyte.
/// This size is supported by all current achetectures (x86_64, AArch64, riscv64).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Size1GiB {}

impl PageSize for Size1GiB {
    const SIZE: u64 = 1073741824;

    fn is_supported() -> bool {
        true
    }
}

/// A page size type for 512 gibibytes.
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
