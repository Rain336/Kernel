use super::utils::read_entry;
use crate::page_flags::{HUGE_PAGE_FLAGS, PAGE_TABLE_FLAGS};
use common::addr::PhysAddr;
use common::memory::{
    DIRECT_MAPPING_LEVEL_3_PAGE_TABLE, DIRECT_MAPPING_SIZE, DIRECT_MAPPING_START,
};
use log::warn;
use memory::addr::VirtAddrExt;
use memory::page_table::{PageTable, PageTableLevel};
use memory::size::{PageSize, Size1GiB, Size2MiB, Size4KiB};

pub fn init_direct_mapping(mut size: u64, mut root: &mut PageTable) {
    debug_assert!(
        !DIRECT_MAPPING_LEVEL_3_PAGE_TABLE.is_initialized(),
        "Direct mapping init function called twice"
    );

    if size > DIRECT_MAPPING_SIZE {
        warn!("Physical memory bigger than direct mapping area");
        size = DIRECT_MAPPING_SIZE;
    }

    let mut level = PageTableLevel::highest();
    while level != PageTableLevel::LEVEL_3 {
        root = unsafe {
            &mut *read_entry(
                &mut root[DIRECT_MAPPING_START.page_table_index(level)],
                PAGE_TABLE_FLAGS,
            )
        };
        level = level.next_lower_level().unwrap();
    }

    assert!(
        DIRECT_MAPPING_LEVEL_3_PAGE_TABLE
            .set(PhysAddr::new_truncate(root as *mut _ as u64))
            .is_ok(),
        "Direct mapping already initialized"
    );

    let mut virt = DIRECT_MAPPING_START;
    let mut phys = PhysAddr::zero();

    while size >= Size1GiB::SIZE {
        root[virt.page_table_index(level)].set_addr(phys, HUGE_PAGE_FLAGS);

        virt += Size1GiB::SIZE;
        phys += Size1GiB::SIZE;
        size -= Size1GiB::SIZE;
    }

    if size == 0 {
        return;
    }

    root = unsafe { &mut *read_entry(&mut root[virt.page_table_index(level)], PAGE_TABLE_FLAGS) };
    level = level.next_lower_level().unwrap();

    while size >= Size2MiB::SIZE {
        root[virt.page_table_index(level)].set_addr(phys, HUGE_PAGE_FLAGS);

        virt += Size2MiB::SIZE;
        phys += Size2MiB::SIZE;
        size -= Size2MiB::SIZE;
    }

    if size == 0 {
        return;
    }

    root = unsafe { &mut *read_entry(&mut root[virt.page_table_index(level)], PAGE_TABLE_FLAGS) };
    level = level.next_lower_level().unwrap();

    while size >= Size4KiB::SIZE {
        root[virt.page_table_index(level)].set_addr(phys, PAGE_TABLE_FLAGS);

        virt += Size4KiB::SIZE;
        phys += Size4KiB::SIZE;
        size -= Size4KiB::SIZE;
    }
}
