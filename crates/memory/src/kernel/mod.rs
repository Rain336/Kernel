//! # The Kernel side of the Memory Subsystem
//!
//! - [`allocator`] Contains the `GlobalAlloc` implementation of the kernel.
//! - [`fixed`] Fixed size allocators used by the kernel.
//! - [`free`] Tracks the free regions of memory.
//! - [`mapping`] Processor-specific mapping for kernel memory.

pub mod allocator;
pub mod fixed;
pub mod free;
pub mod mapping;
