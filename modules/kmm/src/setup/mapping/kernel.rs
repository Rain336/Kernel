use super::utils::read_entry;
use crate::page_flags::PAGE_TABLE_FLAGS;
use common::addr::{PhysAddr, VirtAddr};
use common::memory::{KERNEL_LEVEL_3_PAGE_TABLE, KERNEL_LOAD_START};
use log::warn;
use memory::addr::VirtAddrExt;
use memory::page_table::{PageTable, PageTableFlags, PageTableLevel};

pub fn copy_kernel_entries(mut root: &mut PageTable) {
    debug_assert!(
        !KERNEL_LEVEL_3_PAGE_TABLE.is_initialized(),
        "Kernel page table already set."
    );

    let mut from = unsafe { &mut *read_root_page_table() };
    let mut from_level = PageTableLevel::highest();
    while from_level != PageTableLevel::LEVEL_3 {
        let entry = &mut from[KERNEL_LOAD_START.page_table_index(from_level)];
        if entry.is_unused() {
            return;
        }

        let table = VirtAddr::new_truncate(entry.addr().as_u64()).as_mut_ptr::<PageTable>();
        from = unsafe { &mut *table };
        from_level = from_level.next_lower_level().unwrap();
    }

    let mut level = PageTableLevel::highest();
    while level != PageTableLevel::LEVEL_3 {
        let entry = &mut root[KERNEL_LOAD_START.page_table_index(from_level)];
        root = unsafe { &mut *read_entry(entry, PAGE_TABLE_FLAGS) };
        level = level.next_lower_level().unwrap();
    }

    assert!(
        KERNEL_LEVEL_3_PAGE_TABLE
            .set(PhysAddr::new_truncate(root as *mut _ as u64))
            .is_ok(),
        "Kernel page table already set."
    );

    for i in 508..512 {
        let from_entry = &mut from[i];

        if from_entry.is_unused() {
            continue;
        }

        let to_entry = &mut root[i];

        if cfg!(debug_assertions) && !to_entry.is_unused() {
            warn!(
                "Kernel load page already mapped at {:p} with flags {:x} Overwriting",
                to_entry.addr(),
                to_entry.flags()
            );
        }

        if from_entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            to_entry.set_addr(from_entry.addr(), from_entry.flags());
        } else {
            let table = crate::pmm::allocate().start_address();
            to_entry.set_addr(table, from_entry.flags());

            let from = unsafe {
                &mut *VirtAddr::new_truncate(from_entry.addr().as_u64()).as_mut_ptr::<PageTable>()
            };
            let table =
                unsafe { &mut *VirtAddr::new_truncate(table.as_u64()).as_mut_ptr::<PageTable>() };
            copy_page_table(from, table, PageTableLevel::LEVEL_2);
        }
    }
}

fn copy_page_table(from: &mut PageTable, to: &mut PageTable, level: PageTableLevel) {
    for (from, to) in from.iter().zip(to.iter_mut()) {
        if from.is_unused() {
            continue;
        }

        if from.flags().contains(PageTableFlags::HUGE_PAGE) {
            to.set_addr(from.addr(), from.flags());
        } else if let Some(next) = level.next_lower_level() {
            let table = crate::pmm::allocate().start_address();
            to.set_addr(table, from.flags());

            let from = unsafe {
                &mut *VirtAddr::new_truncate(from.addr().as_u64()).as_mut_ptr::<PageTable>()
            };
            let table =
                unsafe { &mut *VirtAddr::new_truncate(table.as_u64()).as_mut_ptr::<PageTable>() };
            copy_page_table(from, table, next);
        } else {
            to.set_addr(from.addr(), from.flags());
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn read_root_page_table() -> *mut PageTable {
    let (frame, _) = x86_64::registers::control::Cr3::read();
    VirtAddr::new_truncate(frame.start_address().as_u64()).as_mut_ptr::<PageTable>()
}
