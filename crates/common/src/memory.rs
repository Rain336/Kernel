use crate::addr::{PhysAddr, VirtAddr};
use crate::sync::SyncLazy;
use core::sync::atomic::{AtomicBool, Ordering};

static MEMORY_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Gets whenever the memory subsystem is initialized.
#[inline]
pub fn is_initialized() -> bool {
    MEMORY_INITIALIZED.load(Ordering::Relaxed) || MEMORY_INITIALIZED.load(Ordering::Acquire)
}

/// Marks the memory subsystem as initialized.
#[inline]
pub fn set_initialized() {
    MEMORY_INITIALIZED.store(true, Ordering::Release);
}

/// The start of the kernel's direct mapped memory area.
pub const DIRECT_MAPPING_START: VirtAddr = unsafe { VirtAddr::new_unsafe(0xFFFFFF0000000000) };

/// Size of the kernel's direct mapped memory area, in bytes.
/// Guaranteed to be a multiple of 1 GiB.
pub const DIRECT_MAPPING_SIZE: u64 = 4 * 1024 * 1024 * 1024;

/// Converts a physical address into a virtual adress, using the direct mapped memory area.
pub fn physical_to_virtual(phys: PhysAddr) -> VirtAddr {
    debug_assert!(is_initialized(), "The memory subsystem needs to be initialized, before the direct mapped memory area can be used.");
    assert!(
        phys.as_u64() < DIRECT_MAPPING_SIZE,
        "Physical address outside of direct mapped memory area"
    );

    DIRECT_MAPPING_START + phys.as_u64()
}

/// Converts a virtual address to a physical address, by walking the page tables.
#[inline]
pub fn virtual_to_physical(virt: VirtAddr) -> PhysAddr {
    unsafe { crate::magic::virtual_to_physical(virt) }
}

/// Information about the MMU of this system.
pub struct MemoryInfo {
    /// How many bits a virtual address can have.
    pub virtual_address_bits: u64,

    /// How many bits a physical address can have.
    pub physical_address_bits: u64,

    /// Mask to extract the address from a page table entry.
    pub page_table_entry_address_mask: u64,

    /// The highest level of page table supported.
    pub highest_page_table_level: u8,
}

/// Gets the current [`MemoryInfo`].
pub static MEMORY_INFO: SyncLazy<MemoryInfo> =
    SyncLazy::new(|| unsafe { crate::magic::get_memory_info() });
