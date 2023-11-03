// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Translation Module
//!
//! Implements virtual to physical address translation, as implemented by the MMU.
use super::page_table::{LockedPageTable, PageTableLevel, PageTranslation};
use crate::addr::VirtAddrExt;
use common::addr::{PhysAddr, VirtAddr};
use common::memory::{
    physical_to_virtual, DIRECT_MAPPING_SIZE, DIRECT_MAPPING_START, KERNEL_SPACE_START,
};
use common::sync::SyncLazy;

pub const KERNEL_PAGE_TABLE_LEVEL: PageTableLevel = unsafe { PageTableLevel::new_unsafe(3) };

pub static KERNEL_PAGE_TABLE: SyncLazy<&'static LockedPageTable> = SyncLazy::new(|| {
    let mut table =
        unsafe { &*physical_to_virtual(get_root_page_table()).as_ptr::<LockedPageTable>() };
    let mut level = PageTableLevel::highest();

    while level != KERNEL_PAGE_TABLE_LEVEL {
        table = match table[KERNEL_SPACE_START.page_table_index(level)]
            .get()
            .unwrap()
            .translate()
        {
            PageTranslation::PageTable(addr) => unsafe {
                &*physical_to_virtual(addr).as_ptr::<LockedPageTable>()
            },
            _ => panic!("Kernel not mapped?"),
        };
        level = level.next_lower_level().unwrap();
    }

    table
});

/// Maps the given virtual address to a physical address, or `None` if the address isn't mapped.
pub fn virtual_to_physical(addr: VirtAddr) -> Option<PhysAddr> {
    // The address can be in one of three memory areas.
    // The userspace area, which gets mapped using the currently active process address space.
    // The direct mapping area, which just means subtracting the offset.
    // The kernel-space area, which needs to use locked page tables to translate.

    if addr < DIRECT_MAPPING_START {
        todo!("resolve in current address space")
    } else if addr < KERNEL_SPACE_START {
        if addr < (DIRECT_MAPPING_START + DIRECT_MAPPING_SIZE) {
            None
        } else {
            Some(PhysAddr::new_truncate(
                (addr - DIRECT_MAPPING_START).as_u64(),
            ))
        }
    } else {
        translate(addr, &KERNEL_PAGE_TABLE, KERNEL_PAGE_TABLE_LEVEL)
    }
}

/// Resolves the give virtual address `addr` in the given locked page table `table` of level `level`
fn translate(addr: VirtAddr, table: &LockedPageTable, level: PageTableLevel) -> Option<PhysAddr> {
    let next = match table[addr.page_table_index(level)].get()?.translate() {
        PageTranslation::PageTable(next) => next,
        PageTranslation::Page(offset) => {
            return Some(offset + (addr.as_u64() & level.address_space_mask()))
        }
        PageTranslation::None => return None,
    };

    match level.next_lower_level() {
        Some(level) => {
            let table = unsafe { &*physical_to_virtual(next).as_ptr::<LockedPageTable>() };
            translate(addr, table, level)
        }
        None => {
            let offset: u64 = addr.page_offset().into();
            return Some(next + offset);
        }
    }
}
