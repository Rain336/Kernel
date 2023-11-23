// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Kernel Memory Management Setup
//!
//! During initialization two things have to be done:
//! - The memory map has to read in, so that the physical memory manager knows about all free regions of memory available. [`memory_map`]
//! - The mapping described in [`common::memory`] has to be set up. [`mapping`]
//!
mod mapping;
mod memory_map;

use common::memory::{MemoryInfo, MEMORY_INFO};
use interface::ModuleInterface;
use log::debug;

/// Initializes the Kernel Memory Management (KMM) Module.
pub fn init(iface: &ModuleInterface) {
    init_memory_info(iface);
    let size = memory_map::read_memory_map(&iface.memory_map_info);
    mapping::init(size);
}

fn init_memory_info(iface: &ModuleInterface) {
    debug!(
        "Memory Info: VA: {} PA: {} LVL: {}",
        iface.memory_info.virtual_address_bits,
        iface.memory_info.physical_address_bits,
        iface.memory_info.highest_page_table_level
    );

    let ok = MEMORY_INFO
        .set(MemoryInfo {
            virtual_address_bits: iface.memory_info.virtual_address_bits,
            physical_address_bits: iface.memory_info.physical_address_bits,
            page_table_entry_address_mask: iface.memory_info.page_table_entry_address_mask,
            highest_page_table_level: iface.memory_info.highest_page_table_level,
        })
        .is_ok();

    debug_assert!(ok, "Memory info already set.");
}
