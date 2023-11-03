use common::addr::{PhysAddr, VirtAddr};
use common::memory::{physical_to_virtual, KERNEL_DYNAMIC_START, KERNEL_LEVEL_3_PAGE_TABLE};
use memory::addr::VirtAddrExt;
use memory::page_table::{LockedPageTable, PageTableFlags, PageTableLevel};

pub fn translate_address(virt: VirtAddr) -> Option<PhysAddr> {
    debug_assert!(
        virt >= KERNEL_DYNAMIC_START,
        "Address outside of kernel area"
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

        if entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            return Some(entry.addr() + (virt.as_u64() & level.address_space_mask()));
        }

        // SAFETY: `entry.addr()` is an existing page table.
        table = unsafe { &*physical_to_virtual(entry.addr()).as_ptr::<LockedPageTable>() };
        level = level.next_lower_level().unwrap();
    }

    table[virt.page_table_index(level)]
        .get()
        .map(|entry| entry.addr() + u64::from(virt.page_offset()))
}
