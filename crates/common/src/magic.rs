//! # Magic Methods
//!
//! This module implements all magic methods used by the common library.
//! "magic methods" are special functions defined here in common, but implemented by other modules from the kernel.
//! They are resolved during linkage of the kernel.
//!
use crate::addr::{PhysAddr, VirtAddr};

extern "Rust" {
    /// Converts a virtual address inside the userspace area to a physical address, by walking the page tables.
    ///
    /// Implemented By: UMM
    #[link_name = "__internal_virtual_to_physical_user"]
    pub fn virtual_to_physical_user(virt: VirtAddr) -> Option<PhysAddr>;

    /// Converts a virtual address inside the kernel dynamic heap or kernel load area to a physical address, by walking the page tables.
    ///
    /// Implemented By: KMM
    #[link_name = "__internal_virtual_to_physical_kernel"]
    pub fn virtual_to_physical_kernel(virt: VirtAddr) -> Option<PhysAddr>;
}
