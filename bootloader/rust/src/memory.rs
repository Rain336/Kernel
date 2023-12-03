// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use bootloader_api::BootInfo;
use core::arch::x86_64::__cpuid;
use runner::interface::{MemoryInfo, MemoryMapInfo, MemoryMapType};

pub fn get_memory_map_info(info: &BootInfo) -> MemoryMapInfo {
    MemoryMapInfo {
        memory_map: info.memory_regions.as_ptr() as u64,
        memory_map_count: info.memory_regions.len(),
        memory_map_type: MemoryMapType::Rust,
    }
}

pub fn get_memory_info() -> MemoryInfo {
    if has_la57() {
        MemoryInfo {
            virtual_address_bits: 57,
            physical_address_bits: 52,
            page_table_entry_address_mask: 0x000ffffffffff000,
            highest_page_table_level: 5,
        }
    } else {
        MemoryInfo {
            virtual_address_bits: 48,
            physical_address_bits: 52,
            page_table_entry_address_mask: 0x000ffffffffff000,
            highest_page_table_level: 4,
        }
    }
}

fn has_la57() -> bool {
    let result = unsafe { __cpuid(0) };
    if result.eax >= 0x07 {
        let result = unsafe { __cpuid(0x07) };
        (result.ecx & (1 << 16)) != 0
    } else {
        false
    }
}
