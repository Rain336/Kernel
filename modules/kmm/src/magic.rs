// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::addr::{PhysAddr, VirtAddr};

/// Converts a virtual address inside the kernel dynamic heap or kernel load area to a physical address, by walking the page tables.
///
/// Implemented By: KMM
#[export_name = "__internal_virtual_to_physical_kernel"]
fn virtual_to_physical_kernel(virt: VirtAddr) -> Option<PhysAddr> {
    crate::vmm::translate_address(virt)
}

extern "Rust" {
    /// Gets the physical address of the root page table.
    ///
    /// Implemented-By: PI
    #[link_name = "__internal_get_root_page_table"]
    pub fn get_root_page_table() -> PhysAddr;

    /// Sets the physical address of the root page table.
    ///
    /// Implemented-By: PI
    #[link_name = "__internal_set_root_page_table"]
    pub fn set_root_page_table(address: PhysAddr);
}
