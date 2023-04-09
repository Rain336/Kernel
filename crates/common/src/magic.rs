//! # Magic Methods
//!
//! This module implements all magic methods used inside microdragon.
//! "magic methods" are special functions defined here in common, but implementd by other crates from the kernel.
//! They are resolved during linkage of the kernel.

use crate::addr::{PhysAddr, VirtAddr};
use crate::memory::MemoryInfo;

extern "Rust" {
    /// Gathers the required information and creates the [`MemoryInfo`] struct.
    ///
    /// Implemented By: Platform
    #[link_name = "__internal_get_memory_info"]
    pub fn get_memory_info() -> MemoryInfo;

    /// Converts a virtual address to a physical address, by walking the page tables.
    ///
    /// Implemented By: Memory
    #[link_name = "__internal_virtual_to_physical"]
    pub fn virtual_to_physical(virt: VirtAddr) -> PhysAddr;
}
