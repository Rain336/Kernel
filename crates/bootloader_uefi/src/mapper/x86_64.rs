use crate::mapper::{KERNEL_CODE_START, KERNEL_DATA_START};
use alloc::boxed::Box;
use log::debug;
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::frame::PhysFrameRange;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::{Page, PageTable, PageTableFlags, PageTableIndex, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};

fn allocate_page_table() -> &'static mut PageTable {
    Box::leak(Box::new(PageTable::new()))
}

fn map_kernel_page(
    level_2: &mut PageTable,
    page: Page,
    frame: PhysFrame,
    writeable: bool,
    executable: bool,
) {
    let table = if level_2[page.p2_index()].is_unused() {
        let table = allocate_page_table();
        level_2[page.p2_index()].set_addr(
            PhysAddr::new_truncate(table as *const _ as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
        );
        table
    } else {
        let address = level_2[page.p2_index()].addr().as_u64();
        unsafe { &mut *(address as *mut PageTable) }
    };

    let mut flags = PageTableFlags::PRESENT | PageTableFlags::GLOBAL;
    if writeable {
        flags |= PageTableFlags::WRITABLE;
    }
    if !executable {
        flags |= PageTableFlags::NO_EXECUTE;
    }

    table[page.p1_index()].set_addr(frame.start_address(), flags);
}

pub struct MemoryMapper {
    level_4: &'static mut PageTable,
    level_2_code: &'static mut PageTable,
    level_2_data: &'static mut PageTable,
}

impl MemoryMapper {
    pub fn new() -> Self {
        let level_4 = allocate_page_table();
        let level_3 = allocate_page_table();
        let level_2_code = allocate_page_table();
        let level_2_data = allocate_page_table();

        let level_4_address = PhysAddr::new_truncate(level_4 as *const _ as u64);
        level_4[510].set_addr(
            level_4_address,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
        );
        level_4[511].set_addr(
            PhysAddr::new_truncate(level_3 as *const _ as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
        );

        level_3[510].set_addr(
            PhysAddr::new_truncate(level_2_data as *const _ as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
        );
        level_3[511].set_addr(
            PhysAddr::new_truncate(level_2_code as *const _ as u64),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
        );

        MemoryMapper {
            level_4,
            level_2_code,
            level_2_data,
        }
    }

    pub fn map_kernel_code(
        &mut self,
        virtual_adress: u64,
        physical_address: u64,
        pages: u64,
        writeable: bool,
        executable: bool,
    ) {
        assert!(
            virtual_adress >= KERNEL_CODE_START,
            "Page not in kernel code memory"
        );

        let page = PageRange {
            start: Page::containing_address(VirtAddr::new_truncate(virtual_adress)),
            end: Page::containing_address(VirtAddr::new_truncate(virtual_adress + (pages * 4096))),
        };
        let frame = PhysFrameRange {
            start: PhysFrame::containing_address(PhysAddr::new_truncate(physical_address)),
            end: PhysFrame::containing_address(PhysAddr::new_truncate(
                physical_address + (pages * 4096),
            )),
        };

        for (page, frame) in page.zip(frame) {
            debug!(
                "Mapping Page {:#x} into Frame {:#x} Writeable: {} Executable: {}",
                page.start_address(),
                frame.start_address(),
                writeable,
                executable
            );
            map_kernel_page(self.level_2_code, page, frame, writeable, executable);
        }
    }

    pub fn map_kernel_data(
        &mut self,
        virtual_adress: u64,
        physical_address: u64,
        pages: u64,
        writeable: bool,
        executable: bool,
    ) {
        assert!(
            (KERNEL_DATA_START..KERNEL_CODE_START).contains(&virtual_adress),
            "Page not in kernel data memory"
        );
        assert!(
            (virtual_adress + pages * 4096) <= KERNEL_CODE_START,
            "Kernel data memory overlaps into kernel code memory"
        );

        let page = PageRange {
            start: Page::containing_address(VirtAddr::new_truncate(virtual_adress)),
            end: Page::containing_address(VirtAddr::new_truncate(virtual_adress + (pages * 4096))),
        };
        let frame = PhysFrameRange {
            start: PhysFrame::containing_address(PhysAddr::new_truncate(physical_address)),
            end: PhysFrame::containing_address(PhysAddr::new_truncate(
                physical_address + (pages * 4096),
            )),
        };

        for (page, frame) in page.zip(frame) {
            debug!(
                "Mapping Page {:#x} into Frame {:#x} Writeable: {} Executable: {}",
                page.start_address(),
                frame.start_address(),
                writeable,
                executable
            );
            map_kernel_page(self.level_2_data, page, frame, writeable, executable);
        }
    }

    pub fn translate(&self, virtual_adress: u64) -> u64 {
        assert!(
            virtual_adress >= KERNEL_DATA_START,
            "Page not in kernel memory"
        );

        let virtual_adress = VirtAddr::new_truncate(virtual_adress);
        let level_2 = if virtual_adress.p3_index() == PageTableIndex::new_truncate(510) {
            &self.level_2_data
        } else {
            &self.level_2_code
        };

        let table = if level_2[virtual_adress.p2_index()].is_unused() {
            return 0;
        } else {
            let address = level_2[virtual_adress.p2_index()].addr().as_u64();
            unsafe { &*(address as *mut PageTable) }
        };

        let offset: u64 = virtual_adress.page_offset().into();
        table[virtual_adress.p1_index()].addr().as_u64() + offset
    }

    pub fn load(self) {
        unsafe {
            Cr3::write(
                PhysFrame::containing_address(PhysAddr::new_truncate(
                    self.level_4 as *const _ as u64,
                )),
                Cr3Flags::empty(),
            );
        }
    }
}
