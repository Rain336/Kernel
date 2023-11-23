// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::addr::VirtAddr;
use core::ptr;
use log::{debug, info, warn};
use memory::page_table::PageTable;

mod direct;
mod kernel;
mod utils;

pub fn init(size: u64) {
    #[cfg(debug_assertions)]
    if common::memory::is_initialized() {
        warn!("Memory map already initialized");
        return;
    }

    let root = crate::pmm::allocate().start_address();
    debug!("Root page table: {:p}", root);

    let table = VirtAddr::new_truncate(root.as_u64());
    // SAFETY: table is writeable for up to 4096 bytes.
    unsafe { ptr::write_bytes(table.as_mut_ptr::<u8>(), 0, 4096) };

    // SAFETY: table is set to `0` and therefor a valid page table.
    let table = unsafe { &mut *table.as_mut_ptr::<PageTable>() };

    info!("Copying Kernel Load Area entries...");
    kernel::copy_kernel_entries(table);
    info!("Creating Kernel Direct Mapping Area...");
    direct::init_direct_mapping(size, table);

    info!("Switching to kernel page table...");
    unsafe { crate::magic::set_root_page_table(root) };

    #[cfg(debug_assertions)]
    common::memory::set_initialized();
}
