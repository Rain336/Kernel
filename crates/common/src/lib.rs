//! # Common Kernel Library
//!
//! The common kernel library contains constructs and primitives used by all parts of the kernel in an architecture-independent way.
//!
//! - [`addr`] Contains the [`addr::VirtAddr`] and [`addr::PhysAddr`] structs.
//! - [`memory`] defines the memory layout of the kernel and the OS as a whole.
//! - [`sync`] supplies different primitives of synchronization to be used by the kernel.
//!
#![no_std]

pub mod addr;
mod magic;
pub mod memory;
pub mod sync;
