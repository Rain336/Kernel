//! # The Kernel Allocator

use super::fixed;
use super::mapping::{unmap_bytes, MemoryPermissions};
use crate::translation::{physical_to_virtual, KERNEL_OFFSET};
use common::addr::VirtAddr;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::sync::atomic::{AtomicU64, Ordering};

/// All ZSTs get this special adress
const ZST_ADDRESS: *mut u8 = 0xFFFFFFFFDEADBEEF as *mut u8;

#[global_allocator]
static ALLOCATOR: GlobalAllocator = GlobalAllocator;

pub struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let padded = layout.pad_to_align();

        if padded.size() == 0 {
            ZST_ADDRESS
        } else if padded.size() <= 64 {
            fixed::FIXED_64.allocate()
        } else if padded.size() <= 128 {
            fixed::FIXED_128.allocate()
        } else if padded.size() <= 256 {
            fixed::FIXED_256.allocate()
        } else if padded.size() <= 512 {
            fixed::FIXED_512.allocate()
        } else {
            allocate_bytes(padded.size() as u64)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let padded = layout.pad_to_align();

        let result = if padded.size() == 0 {
            true
        } else if padded.size() <= 64 {
            fixed::FIXED_64.free(ptr)
        } else if padded.size() <= 128 {
            fixed::FIXED_128.free(ptr)
        } else if padded.size() <= 256 {
            fixed::FIXED_256.free(ptr)
        } else if padded.size() <= 512 {
            fixed::FIXED_512.free(ptr)
        } else {
            free_bytes(ptr, padded.size() as u64)
        };

        debug_assert!(
            result,
            "Cannot free pointer {:p}, since it wasn't allocated using GlobalAlloc",
            ptr
        );
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let padded = layout.pad_to_align();
        let new_padded =
            unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) }.pad_to_align();

        if padded.size() == 0 {
            self.alloc(new_padded)
        } else if padded.size() <= 64 {
            if new_padded.size() <= 64 {
                ptr
            } else {
                let result = if new_padded.size() <= 128 {
                    fixed::FIXED_128.allocate()
                } else if new_padded.size() <= 256 {
                    fixed::FIXED_256.allocate()
                } else if new_padded.size() <= 512 {
                    fixed::FIXED_512.allocate()
                } else {
                    allocate_bytes(new_padded.size() as u64)
                };

                unsafe { ptr::copy_nonoverlapping(ptr, result, padded.size()) };

                fixed::FIXED_64.free(ptr);
                result
            }
        } else if padded.size() <= 128 {
            if new_padded.size() <= 128 {
                ptr
            } else {
                let result = if new_padded.size() <= 256 {
                    fixed::FIXED_256.allocate()
                } else if new_padded.size() <= 512 {
                    fixed::FIXED_512.allocate()
                } else {
                    allocate_bytes(new_padded.size() as u64)
                };

                unsafe { ptr::copy_nonoverlapping(ptr, result, padded.size()) };

                fixed::FIXED_128.free(ptr);
                result
            }
        } else if padded.size() <= 256 {
            if new_padded.size() <= 256 {
                ptr
            } else {
                let result = if new_padded.size() <= 512 {
                    fixed::FIXED_512.allocate()
                } else {
                    allocate_bytes(new_padded.size() as u64)
                };

                unsafe { ptr::copy_nonoverlapping(ptr, result, padded.size()) };

                fixed::FIXED_256.free(ptr);
                result
            }
        } else if padded.size() <= 512 {
            if new_padded.size() <= 512 {
                ptr
            } else {
                let result = allocate_bytes(new_padded.size() as u64);

                unsafe { ptr::copy_nonoverlapping(ptr, result, padded.size()) };

                fixed::FIXED_512.free(ptr);
                result
            }
        } else if bytes_to_pages(padded.size() as u64) == bytes_to_pages(new_padded.size() as u64) {
            ptr
        } else {
            let result = allocate_bytes(new_padded.size() as u64);
            unsafe { ptr::copy_nonoverlapping(ptr, result, padded.size()) };
            free_bytes(ptr, padded.size() as u64);
            result
        }
    }
}

const fn bytes_to_pages(bytes: u64) -> u64 {
    (bytes + 4095) >> 12
}

static KERNEL_DATA_POINTER: AtomicU64 = AtomicU64::new(KERNEL_OFFSET.as_u64() + 0x40000000);

fn allocate_bytes(bytes: u64) -> *mut u8 {
    let pages = bytes_to_pages(bytes);
    if pages == 1 {
        let page = super::free::allocate_page().unwrap();
        physical_to_virtual(page).as_mut_ptr::<u8>()
    } else {
        let mut result = None;

        super::free::allocate(bytes, |phys, bytes| {
            let virt =
                VirtAddr::new_truncate(KERNEL_DATA_POINTER.fetch_add(bytes, Ordering::AcqRel));
            result.get_or_insert(virt);
            super::mapping::map_bytes(virt, phys, bytes, MemoryPermissions::ReadWrite).unwrap();
        });

        result.unwrap().as_mut_ptr::<u8>()
    }
}

fn free_bytes(ptr: *mut u8, bytes: u64) -> bool {
    let virt: VirtAddr = ptr.into();
    unmap_bytes(virt, bytes, super::free::free);
    true
}
