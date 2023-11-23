// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use limine::{MemmapRequest, PagingMode, PagingModeRequest};
use runner::interface::{MemoryInfo, MemoryMapInfo, MemoryMapType};

static MEMORY_MAP_REQUEST: MemmapRequest = MemmapRequest::new(0);

pub fn get_memory_map_info() -> MemoryMapInfo {
    let response = MEMORY_MAP_REQUEST
        .get_response()
        .get()
        .expect("No memory map provided by the bootloader");

    MemoryMapInfo {
        memory_map: response.entries.as_ptr() as u64,
        memory_map_count: response.entry_count as usize,
        memory_map_type: MemoryMapType::Limine,
    }
}

#[cfg(target_arch = "riscv64")]
const PAGING_MODE: PagingMode = PagingMode::Sv57;
#[cfg(not(target_arch = "riscv64"))]
const PAGING_MODE: PagingMode = PagingMode::Lvl5;

static PAGING_MODE_REQUEST: PagingModeRequest = PagingModeRequest::new(0).mode(PAGING_MODE);

pub fn get_memory_info() -> MemoryInfo {
    let response = PAGING_MODE_REQUEST
        .get_response()
        .get()
        .expect("No memory info supplied");

    #[cfg(not(target_arch = "riscv64"))]
    match response.mode {
        PagingMode::Lvl4 => MemoryInfo {
            virtual_address_bits: 48,
            physical_address_bits: 52,
            page_table_entry_address_mask: 0x000ffffffffff000,
            highest_page_table_level: 4,
        },
        PagingMode::Lvl5 => MemoryInfo {
            virtual_address_bits: 57,
            physical_address_bits: 52,
            page_table_entry_address_mask: 0x000ffffffffff000,
            highest_page_table_level: 5,
        },
    }
}
