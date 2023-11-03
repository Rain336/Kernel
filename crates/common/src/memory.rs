//! # Memory Module
//!
//! |Address Range|Usage|
//! |:-:|:-|
//! |0x0000000000000000 - 0xFFFFFF0000000000| Userspace Area |
//! |0xFFFFFF0000000000 - 0xFFFFFF8000000000| Kernel Direct Mapping Area |
//! |0xFFFFFF8000000000 - 0xFFFFFFFF80000000| Kernel Dynamic Heap Area |
//! |0xFFFFFFFF80000000 - 0xFFFFFFFFFFFFFFFF| Kernel Load Area |
//!
#![allow(clippy::unusual_byte_groupings)]

use crate::addr::{PhysAddr, VirtAddr};
use crate::sync::SyncOnceCell;
#[cfg(debug_assertions)]
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(debug_assertions)]
static MEMORY_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Gets whenever the memory subsystem is initialized.
#[inline]
#[cfg(debug_assertions)]
pub fn is_initialized() -> bool {
    MEMORY_INITIALIZED.load(Ordering::Relaxed) || MEMORY_INITIALIZED.load(Ordering::Acquire)
}

/// Marks the memory subsystem as initialized.
#[inline]
#[cfg(debug_assertions)]
pub fn set_initialized() {
    MEMORY_INITIALIZED.store(true, Ordering::Release);
}

/// The start of the kernel's direct mapped memory area.
pub const DIRECT_MAPPING_START: VirtAddr = VirtAddr::new_const(0xFFFFFF0000000000);

/// The end of the kernel's direct mapped memory area.
pub const DIRECT_MAPPING_END: VirtAddr = VirtAddr::new_const(0xFFFFFF8000000000);

/// Size of the kernel's direct mapped memory area, in bytes.
/// Guaranteed to be a multiple of 1 GiB.
pub const DIRECT_MAPPING_SIZE: u64 = DIRECT_MAPPING_END.as_u64() - DIRECT_MAPPING_START.as_u64();

/// Physical address of the level 3 page table for direct mapping.
pub static DIRECT_MAPPING_LEVEL_3_PAGE_TABLE: SyncOnceCell<PhysAddr> = SyncOnceCell::new();

/// The start of the kernel's dynamic heap area.
pub const KERNEL_DYNAMIC_START: VirtAddr = VirtAddr::new_const(0xFFFFFF8000000000);

/// The end of the kernel's dynamic heap area.
pub const KERNEL_DYNAMIC_END: VirtAddr = VirtAddr::new_const(0xFFFFFFFF80000000);

/// Size of the kernel's dynamic heap area, in bytes.
pub const KERNEL_DYNAMIC_HEAP_SIZE: u64 =
    KERNEL_DYNAMIC_END.as_u64() - KERNEL_DYNAMIC_START.as_u64();

/// The start of the kernel's load area.
pub const KERNEL_LOAD_START: VirtAddr = VirtAddr::new_const(0xFFFFFFFF80000000);

/// The end of the kernel's load area.
pub const KERNEL_LOAD_END: VirtAddr = VirtAddr::new_const(0xFFFFFFFFFFFFFFFF);

/// Size of the kernel's load area, in bytes.
pub const KERNEL_LOAD_SIZE: u64 = KERNEL_LOAD_END.as_u64() - KERNEL_LOAD_START.as_u64();

/// Physical address of the level 3 page table for kernel areas.
pub static KERNEL_LEVEL_3_PAGE_TABLE: SyncOnceCell<PhysAddr> = SyncOnceCell::new();

/// Converts a physical address into a virtual address, using the direct mapped memory area.
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
pub fn virtual_to_physical(virt: VirtAddr) -> Option<PhysAddr> {
    if virt < DIRECT_MAPPING_START {
        unsafe { crate::magic::virtual_to_physical_user(virt) }
    } else if virt < KERNEL_DYNAMIC_START {
        Some(PhysAddr::new_truncate(
            (virt - DIRECT_MAPPING_START).as_u64(),
        ))
    } else {
        unsafe { crate::magic::virtual_to_physical_kernel(virt) }
    }
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
pub static MEMORY_INFO: SyncOnceCell<MemoryInfo> = SyncOnceCell::new();

/// Gets the current [`MemoryInfo`] as a reference.
pub fn get_memory_info() -> &'static MemoryInfo {
    MEMORY_INFO.get().expect("Memory Info missing")
}
