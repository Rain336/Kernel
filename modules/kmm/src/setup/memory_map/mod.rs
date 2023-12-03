// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Memory Map Setup
//!
//! Reads in the memory map, provides the [`crate::pmm`] with free physical frames and returns the highest physical memory address.
//! The boot loader provides the memory map in different formats, each having a separate sub-module to this one.
//!
mod limine;
mod rust;

use interface::{MemoryMapInfo, MemoryMapType};

const RESERVED_AREA: u64 = 1024 * 1024;

/// Reads in the memory map from the boot loader, passes the free regions to the PMM and returns the highest physical memory address.
pub fn read_memory_map(map: &MemoryMapInfo) -> u64 {
    debug_assert!(
        !crate::pmm::is_initialized(),
        "Physical memory manager already initialized"
    );

    match map.memory_map_type {
        MemoryMapType::Limine => limine::read_limine_memory_map(map),
        MemoryMapType::Rust => rust::read_rust_memory_map(map),
    }
}
