// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::page_flags::PAGE_TABLE_FLAGS;
use common::memory::{
    physical_to_virtual, KERNEL_DYNAMIC_END, KERNEL_DYNAMIC_START, KERNEL_LEVEL_3_PAGE_TABLE,
};
use core::ptr;
use memory::frame::PhysFrame;
use memory::page::Page;
use memory::page_table::{LockedPageTable, PageTableEntry, PageTableLevel};

/// Maps the given page into the given frame. The page has to be in the kernel dynamic heap area.
pub fn map_page(phys: PhysFrame, virt: Page) {
    debug_assert!(
        virt.start_address() >= KERNEL_DYNAMIC_START
            && virt.start_address() + 4096u64 <= KERNEL_DYNAMIC_END,
        "Page outside of kernel dynamic heap area"
    );

    let Some(table) = KERNEL_LEVEL_3_PAGE_TABLE.get() else {
        return;
    };
    // SAFETY: table is a valid physical address to a level 3 page table and `physical_to_virtual` converts it to a virtual address we can read/write.
    let mut table = unsafe { &*physical_to_virtual(*table).as_ptr::<LockedPageTable>() };

    let mut level = PageTableLevel::LEVEL_3;
    while !level.is_last() {
        let entry = table[virt.page_table_index(level)].get_or_init(|| {
            let table = crate::pmm::allocate().start_address();

            // SAFETY: table is a 4096 byte memory page.
            unsafe { ptr::write_bytes(physical_to_virtual(table).as_mut_ptr::<u8>(), 0, 4096) };

            PageTableEntry::new(table, PAGE_TABLE_FLAGS)
        });

        // SAFETY: `entry.addr()` is either an existing page table or a newly allocated one.
        table = unsafe { &*physical_to_virtual(entry.addr()).as_ptr::<LockedPageTable>() };
        level = level.next_lower_level().unwrap();
    }

    assert!(
        table[virt.page_table_index(level)]
            .set(PageTableEntry::new(phys.start_address(), PAGE_TABLE_FLAGS))
            .is_ok(),
        "Page at virtual address {:p} already mapped",
        virt.start_address()
    );
}

/// Unmaps the given page. The page has to be in the kernel dynamic heap area.
/// Returns the [`PhysFrame`] the page was mapped into, if it was mapped.
pub fn unmap_page(virt: Page) -> Option<PhysFrame> {
    debug_assert!(
        virt.start_address() >= KERNEL_DYNAMIC_START
            && virt.start_address() + 4096u64 <= KERNEL_DYNAMIC_END,
        "Page outside of kernel dynamic heap area"
    );

    let Some(table) = KERNEL_LEVEL_3_PAGE_TABLE.get() else {
        return None;
    };
    // SAFETY: table is a valid physical address to a level 3 page table and `physical_to_virtual` converts it to a virtual address we can read/write.
    let mut table = unsafe { &*physical_to_virtual(*table).as_ptr::<LockedPageTable>() };

    let mut level = PageTableLevel::LEVEL_3;
    while !level.is_last() {
        let Some(entry) = table[virt.page_table_index(level)].get() else {
            return None;
        };

        // SAFETY: `entry.addr()` is an existing page table.
        table = unsafe { &*physical_to_virtual(entry.addr()).as_ptr::<LockedPageTable>() };
        level = level.next_lower_level().unwrap();
    }

    let old = table[virt.page_table_index(level)].set_unused();
    Some(PhysFrame::containing_address(old.addr()))
}
