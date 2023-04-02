use super::addr::{PhysAddr, VirtAddr};
use super::page_table::{LockedPageTable, PageTable, PageTableLevel, PageTranslation};
use common::sync::SyncLazy;

pub const PHYSICAL_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0xFFFFFF0000000000) };
pub const KERNEL_OFFSET: VirtAddr = unsafe { VirtAddr::new_unsafe(0xFFFFFF8000000000) };
pub const KERNEL_PAGE_TABLE_LEVEL: PageTableLevel = unsafe { PageTableLevel::new_unsafe(3) };

pub static KERNEL_PAGE_TABLE: SyncLazy<&'static LockedPageTable> = SyncLazy::new(|| {
    let mut table =
        unsafe { &*physical_to_virtual(get_root_page_table()).as_ptr::<LockedPageTable>() };
    let mut level = PageTableLevel::highest();

    while level != KERNEL_PAGE_TABLE_LEVEL {
        table = match table[KERNEL_OFFSET.page_table_index(level)]
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

pub fn physical_to_virtual(addr: PhysAddr) -> VirtAddr {
    let result = PHYSICAL_OFFSET + addr.as_u64();
    if result >= KERNEL_OFFSET {
        panic!("physical_to_virtual can only access up to 512 GiB");
    }
    result
}

pub fn virtual_to_physical(addr: VirtAddr) -> Option<PhysAddr> {
    if addr < PHYSICAL_OFFSET {
        let table = get_root_page_table();
        let table = unsafe { &*physical_to_virtual(table).as_ptr::<PageTable>() };
        treverse(addr, table, PageTableLevel::highest())
    } else if addr < KERNEL_OFFSET {
        Some(PhysAddr::new_truncate((addr - PHYSICAL_OFFSET).as_u64()))
    } else {
        treverse_locked(addr, &KERNEL_PAGE_TABLE, KERNEL_PAGE_TABLE_LEVEL)
    }
}

fn treverse(addr: VirtAddr, mut table: &PageTable, mut level: PageTableLevel) -> Option<PhysAddr> {
    loop {
        let next = match table[addr.page_table_index(level)].translate() {
            PageTranslation::PageTable(next) => next,
            PageTranslation::Page(offset) => {
                return Some(offset + (addr.as_u64() & level.address_space_mask()))
            }
            PageTranslation::None => return None,
        };

        match level.next_lower_level() {
            Some(x) => {
                level = x;
                table = unsafe { &*physical_to_virtual(next).as_ptr::<PageTable>() };
            }
            None => {
                let offset: u64 = addr.page_offset().into();
                return Some(next + offset);
            }
        }
    }
}

fn treverse_locked(
    addr: VirtAddr,
    mut table: &LockedPageTable,
    mut level: PageTableLevel,
) -> Option<PhysAddr> {
    loop {
        let next = match table[addr.page_table_index(level)].get()?.translate() {
            PageTranslation::PageTable(next) => next,
            PageTranslation::Page(offset) => {
                return Some(offset + (addr.as_u64() & level.address_space_mask()))
            }
            PageTranslation::None => return None,
        };

        match level.next_lower_level() {
            Some(x) => {
                level = x;
                table = unsafe { &*physical_to_virtual(next).as_ptr::<LockedPageTable>() };
            }
            None => {
                let offset: u64 = addr.page_offset().into();
                return Some(next + offset);
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn get_root_page_table() -> PhysAddr {
    let (table, _) = x86_64::registers::control::Cr3::read();
    table.start_address().into()
}
