//! # Paged Allocator
//!
//! An allocator that allocates pages from the PMM and maps them into the kernel dynamic heap area.
//!
use common::addr::VirtAddr;
use common::memory::{KERNEL_DYNAMIC_END, KERNEL_DYNAMIC_START};
use core::sync::atomic::{AtomicU64, Ordering};
use memory::page::{Page, PageRange};

static VIRTUAL_MEMORY_POINTER: AtomicU64 = AtomicU64::new(KERNEL_DYNAMIC_START.as_u64());

/// Allocates the given amount of pages.
pub fn allocate(pages: usize) -> *mut u8 {
    let size = pages as u64 * 4096;
    let start = VirtAddr::new_const(VIRTUAL_MEMORY_POINTER.fetch_add(size, Ordering::AcqRel));
    let end = start + size;

    assert!(
        end < KERNEL_DYNAMIC_END,
        "Kernel dynamic heap area exhausted"
    );

    let range = PageRange {
        start: Page::containing_address(start),
        end: Page::containing_address(end),
    };
    for page in range {
        crate::vmm::map_page(crate::pmm::allocate(), page);
    }

    start.as_mut_ptr::<u8>()
}

/// Frees the given amount of pages allocated using [`allocate`].
pub fn free(ptr: *mut u8, pages: usize) {
    let size = pages as u64 * 4096;
    // SAFETY: ptr is a valid virtual address.
    let start = unsafe { VirtAddr::new_unsafe(ptr as u64) };
    let end = start + size;

    let range = PageRange {
        start: Page::containing_address(start),
        end: Page::containing_address(end),
    };
    for page in range.rev() {
        if let Some(frame) = crate::vmm::unmap_page(page) {
            crate::pmm::free(frame)
        }
    }
}
