use crate::addr::VirtAddrExt;
use crate::page_table::{LockedPageTable, PageTableEntry, PageTableFlags, PageTranslation};
use crate::size::{PageSize, Size1GiB, Size2MiB, Size4KiB};
use crate::translation::{KERNEL_PAGE_TABLE, KERNEL_PAGE_TABLE_LEVEL};
use common::addr::{PhysAddr, VirtAddr};
use common::memory::physical_to_virtual;
use core::ptr;

const KERNEL_PAGE_TABLE_FLAGS: PageTableFlags = PageTableFlags::VALID
    .union(PageTableFlags::WRITEABLE)
    .union(PageTableFlags::GLOBAL);

#[derive(Clone, Copy)]
pub enum MemoryPermissions {
    Read,
    ReadWrite,
    ReadExecute,
}

impl MemoryPermissions {
    #[cfg(target_arch = "x86_64")]
    const fn flags(self) -> PageTableFlags {
        match self {
            MemoryPermissions::Read => PageTableFlags::NO_EXECUTE,
            MemoryPermissions::ReadWrite => {
                PageTableFlags::NO_EXECUTE.union(PageTableFlags::WRITEABLE)
            }
            MemoryPermissions::ReadExecute => PageTableFlags::empty(),
        }
    }

    #[cfg(riscv)]
    const fn flags(self) -> PageTableFlags {
        match self {
            MemoryPermissions::Read => PageTableFlags::READ,
            MemoryPermissions::ReadWrite => PageTableFlags::READ.union(PageTableFlags::WRITE),
            MemoryPermissions::ReadExecute => PageTableFlags::READ.union(PageTableFlags::EXECUTE),
        }
    }

    #[cfg(target_arch = "x86_64")]
    const fn huge_page_flags(self) -> PageTableFlags {
        self.page_flags().union(PageTableFlags::HUGE_PAGE)
    }

    #[cfg(not(target_arch = "x86_64"))]
    const fn huge_page_flags(self) -> PageTableFlags {
        self.page_flags()
    }

    const fn page_flags(self) -> PageTableFlags {
        PageTableFlags::VALID
            .union(PageTableFlags::GLOBAL)
            .union(self.flags())
    }
}

#[derive(Debug)]
pub enum MappingError {
    RegionAlreadyMapped,
    InvaildPageTable,
}

pub fn map_bytes(
    mut virt: VirtAddr,
    mut phys: PhysAddr,
    mut bytes: u64,
    permissions: MemoryPermissions,
) -> Result<(), MappingError> {
    bytes = (bytes + 4095) & !0xFFF;

    while bytes != 0 {
        if can_map::<Size1GiB>(virt, phys, bytes) {
            if KERNEL_PAGE_TABLE[virt.page_table_index(KERNEL_PAGE_TABLE_LEVEL)]
                .set(PageTableEntry::new(phys, permissions.huge_page_flags()))
                .is_err()
            {
                return Err(MappingError::RegionAlreadyMapped);
            }

            virt += Size1GiB::SIZE;
            phys += Size1GiB::SIZE;
            bytes -= Size1GiB::SIZE;
            continue;
        }

        let level_2 = KERNEL_PAGE_TABLE[virt.page_table_index(KERNEL_PAGE_TABLE_LEVEL)]
            .get_or_init(create_page_table);
        let level_2 = match level_2.translate() {
            PageTranslation::PageTable(addr) => unsafe {
                &*physical_to_virtual(addr).as_ptr::<LockedPageTable>()
            },
            PageTranslation::Page(_) => return Err(MappingError::RegionAlreadyMapped),
            PageTranslation::None => unreachable!("Page Table Entry should be initialized"),
        };
        let level = KERNEL_PAGE_TABLE_LEVEL.next_lower_level().unwrap();

        if can_map::<Size2MiB>(virt, phys, bytes) {
            if level_2[virt.page_table_index(level)]
                .set(PageTableEntry::new(phys, permissions.huge_page_flags()))
                .is_err()
            {
                return Err(MappingError::RegionAlreadyMapped);
            }

            virt += Size2MiB::SIZE;
            phys += Size2MiB::SIZE;
            bytes -= Size2MiB::SIZE;
            continue;
        }

        let level_1 = level_2[virt.page_table_index(level)].get_or_init(create_page_table);
        let level_1 = match level_1.translate() {
            PageTranslation::PageTable(addr) => unsafe {
                &*physical_to_virtual(addr).as_ptr::<LockedPageTable>()
            },
            PageTranslation::Page(_) => return Err(MappingError::RegionAlreadyMapped),
            PageTranslation::None => unreachable!("Page Table Entry should be initialized"),
        };
        let level = level.next_lower_level().unwrap();

        if level_1[virt.page_table_index(level)]
            .set(PageTableEntry::new(phys, permissions.page_flags()))
            .is_err()
        {
            return Err(MappingError::RegionAlreadyMapped);
        }

        virt += Size4KiB::SIZE;
        phys += Size4KiB::SIZE;
        bytes -= Size4KiB::SIZE;
    }

    Ok(())
}

fn can_map<S: PageSize>(virt: VirtAddr, phys: PhysAddr, bytes: u64) -> bool {
    bytes >= S::SIZE && phys.is_aligned(S::SIZE) && virt.is_aligned(S::SIZE)
}

fn create_page_table() -> PageTableEntry {
    let page = super::free::allocate_page().unwrap();
    unsafe { ptr::write_bytes(physical_to_virtual(page).as_mut_ptr::<u8>(), 0, 4096) };
    PageTableEntry::new(page, KERNEL_PAGE_TABLE_FLAGS)
}

pub fn unmap_bytes(mut virt: VirtAddr, mut bytes: u64, f: impl Fn(PhysAddr, u64)) {
    bytes = (bytes + 4095) & !0xFFF;

    while bytes != 0 {
        let level_2 = match KERNEL_PAGE_TABLE[virt.page_table_index(KERNEL_PAGE_TABLE_LEVEL)].get()
        {
            Some(x) => x,
            None => return,
        };
        let level_2 = match level_2.translate() {
            PageTranslation::PageTable(x) => unsafe {
                &*physical_to_virtual(x).as_ptr::<LockedPageTable>()
            },
            PageTranslation::Page(_) => {
                if can_unmap::<Size1GiB>(virt, bytes) {
                    let addr = KERNEL_PAGE_TABLE[virt.page_table_index(KERNEL_PAGE_TABLE_LEVEL)]
                        .set_unused()
                        .addr();
                    f(addr, Size1GiB::SIZE);

                    virt += Size1GiB::SIZE;
                    bytes -= Size1GiB::SIZE;
                    continue;
                } else {
                    return;
                }
            }
            PageTranslation::None => return,
        };

        let level = KERNEL_PAGE_TABLE_LEVEL.next_lower_level().unwrap();
        let level_1 = match level_2[virt.page_table_index(level)].get() {
            Some(x) => x,
            None => return,
        };
        let level_1 = match level_1.translate() {
            PageTranslation::PageTable(x) => unsafe {
                &*physical_to_virtual(x).as_ptr::<LockedPageTable>()
            },
            PageTranslation::Page(_) => {
                if can_unmap::<Size2MiB>(virt, bytes) {
                    let addr = level_2[virt.page_table_index(level)].set_unused().addr();
                    f(addr, Size2MiB::SIZE);

                    virt += Size2MiB::SIZE;
                    bytes -= Size2MiB::SIZE;
                    continue;
                } else {
                    return;
                }
            }
            PageTranslation::None => return,
        };

        let level = level.next_lower_level().unwrap();
        let addr = level_1[virt.page_table_index(level)].set_unused().addr();
        f(addr, Size4KiB::SIZE);

        virt += Size4KiB::SIZE;
        bytes -= Size4KiB::SIZE;
    }
}

fn can_unmap<S: PageSize>(virt: VirtAddr, bytes: u64) -> bool {
    bytes >= S::SIZE && virt.is_aligned(S::SIZE)
}
