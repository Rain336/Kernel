use common::addr::{PhysAddr, VirtAddr};
use core::ptr;
use log::{debug, info, warn};
use memory::page_table::PageTable;

mod direct;
mod kernel;
mod utils;

pub fn init(size: u64) {
    #[cfg(debug_assertions)]
    if common::memory::is_initialized() {
        warn!("Memory map already initialized");
        return;
    }

    let root = crate::pmm::allocate().start_address();
    debug!("Root page table: {:p}", root);

    let table = VirtAddr::new_truncate(root.as_u64());
    // SAFETY: table is writeable for up to 4096 bytes.
    unsafe { ptr::write_bytes(table.as_mut_ptr::<u8>(), 0, 4096) };

    // SAFETY: table is set to `0` and therefor a valid page table.
    let table = unsafe { &mut *table.as_mut_ptr::<PageTable>() };

    info!("Copying Kernel Load Area entries...");
    kernel::copy_kernel_entries(table);
    info!("Creating Kernel Direct Mapping Area...");
    direct::init_direct_mapping(size, table);

    info!("Switching to kernel page table...");
    set_root_page_table(root);

    #[cfg(debug_assertions)]
    common::memory::set_initialized();
}

#[cfg(target_arch = "x86_64")]
fn set_root_page_table(root: PhysAddr) {
    use x86_64::registers::control::Cr3;
    use x86_64::structures::paging::PhysFrame;

    let (_, flags) = Cr3::read();
    unsafe { Cr3::write(PhysFrame::containing_address(root.into()), flags) };
}
