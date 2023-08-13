//! ## The Kernel Memory Management Module (KMM)
//!
//! - [`allocator`] Contains the `GlobalAlloc` implementation of the kernel.
//! - [`fixed`] Fixed size allocators used by the kernel.
//! - [`free`] Tracks the free regions of memory.
//! - [`mapping`] Processor-specific mapping for kernel memory.
#![no_std]

extern crate alloc;

mod allocator;
mod fixed;
mod free;
mod memory_map;
mod platform;
mod setup;

#[global_allocator]
static ALLOCATOR: allocator::GlobalAllocator = allocator::GlobalAllocator;

pub fn init() {
    memory_map::read_memory_map();
}
