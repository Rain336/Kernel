//! # Common Kernel Library
//!
//! The common kernel library contins difrent parts of the kernel that are not architecture specific or
//! only in small parts that can easily be gated using `#[cfg]`.
//!
//! - [`addr`] Contains the [`addr::VirtAddr`] and [`addr::PhysAddr`] structs.
//! - [`memory`] Contains the [`memory::MemoryInfo`] struct with information about the installed MMU.
//! - [`sync`] Primitives of syncronization.
#![no_std]

pub mod addr;
mod magic;
pub mod memory;
pub mod sync;
