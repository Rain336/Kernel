use common::addr::{PhysAddr, VirtAddr};
use common::memory::DIRECT_MAPPING_START;
use memory::addr::VirtAddrExt;
use memory::page_table::{LockedPageTable, PageTableEntry, PageTableFlags, PageTableLevel};

static ROOT_PAGE_TABLE: LockedPageTable = LockedPageTable::new();

#[derive(Clone, Copy)]
pub enum MemoryPermissions {
    Read,
    ReadWrite,
    ReadExecute,
}

impl MemoryPermissions {
    #[cfg(target_arch = "x86_64")]
    const fn translate(self) -> PageTableFlags {
        PageTableFlags::VALID.union(match self {
            MemoryPermissions::Read => PageTableFlags::NO_EXECUTE,
            MemoryPermissions::ReadWrite => {
                PageTableFlags::NO_EXECUTE.union(PageTableFlags::WRITEABLE)
            }
            MemoryPermissions::ReadExecute => PageTableFlags::empty(),
        })
    }

    #[cfg(riscv)]
    const fn translate(self) -> PageTableFlags {
        PageTableFlags::VALID.union(match self {
            MemoryPermissions::Read => PageTableFlags::READ,
            MemoryPermissions::ReadWrite => PageTableFlags::READ.union(PageTableFlags::WRITE),
            MemoryPermissions::ReadExecute => PageTableFlags::READ.union(PageTableFlags::EXECUTE),
        })
    }

    #[cfg(target_arch = "x86_64")]
    const fn translate_huge(self) -> PageTableFlags {
        self.translate_page().union(PageTableFlags::HUGE_PAGE)
    }

    #[cfg(not(target_arch = "x86_64"))]
    const fn translate_huge(self) -> PageTableFlags {
        self.translate_page()
    }

    const fn translate_page(self) -> PageTableFlags {
        self.translate().union(PageTableFlags::GLOBAL)
    }
}

pub fn setup_direct_mapping() {
    fn early_physical_to_virtual(phys: PhysAddr) -> VirtAddr {
        VirtAddr::new_unsafe(phys.as_u64())
    }

    let mut level = PageTableLevel::highest();
    let mut table = &ROOT_PAGE_TABLE;

    while level.as_u8() > 4 {
        let entry = &table[DIRECT_MAPPING_START.page_table_index(level)].get_or_init(|| {
            let page = crate::free::allocate_page().unwrap();
            unsafe {
                early_physical_to_virtual(page)
                    .as_mut_ptr::<LockedPageTable>()
                    .write_bytes(0, core::mem::size_of::<LockedPageTable>());
            }
            PageTableEntry::new(page, MemoryPermissions::ReadWrite.translate_page())
        });

        table = unsafe { &*early_physical_to_virtual(entry.addr()).as_ptr::<LockedPageTable>() };
        level = level.next_lower_level().unwrap();
    }

    table[DIRECT_MAPPING_START.page_table_index(level)].set(PageTableEntry::new(
        PhysAddr::zero(),
        MemoryPermissions::ReadWrite.translate_huge(),
    ));
}
