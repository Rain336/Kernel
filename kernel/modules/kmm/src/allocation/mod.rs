// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Memory Allocator for Kernel Heap Memory
//!
//! This module defines a [`GlobalAlloc`] implementation for the Kernel,
//! using the fixed allocators from the [`fixed`] module for small sub-page size allocations
//! and the page allocator from [`paged`] for page sized and bigger allocations.
//!
mod fixed;
mod paged;

use common::addr::VirtAddr;
use core::alloc::{GlobalAlloc, Layout};
use core::cmp;

/// All ZSTs get this special virtual address
/// ZSTs should not be read or written to, so a mnemonic address makes it easy to deduce the problem, if it does happen.
pub const ZST_ADDRESS: VirtAddr = VirtAddr::new_const(0xFFFFFFFFDEADBEEF);

/// The [`GlobalAlloc`] implementation for the kernel.
pub struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout = layout.pad_to_align();
        let class = SizeClass::new(layout.size());

        class.allocate(layout.size())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let layout = layout.pad_to_align();
        let class = SizeClass::new(layout.size());

        class.free(ptr, layout.size())
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_size = layout.pad_to_align().size();
        let old = SizeClass::new(old_size);
        // SAFETY: ensured by the safeties provided by realloc.
        let new = SizeClass::new(
            unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) }
                .pad_to_align()
                .size(),
        );

        if old == new {
            return ptr;
        }

        let result = new.allocate(new_size);
        // SAFETY: result is valid for at least new_size writes.
        unsafe { result.copy_from_nonoverlapping(ptr, cmp::min(layout.size(), new_size)) };
        old.free(ptr, old_size);

        result
    }
}

/// The size class an allocation can be in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SizeClass {
    /// Size class for ZSTs
    Size0,
    /// 64 bytes or smaller size class
    Size64,
    /// 65-128 bytes size class
    Size128,
    /// 129-256 bytes size class
    Size256,
    /// 257-512 bytes size class
    Size512,
    /// 513 bytes or bigger size class
    PageSize,
}

impl SizeClass {
    /// Determents the size class for the given size
    pub fn new(size: usize) -> Self {
        if size == 0 {
            SizeClass::Size0
        } else if size <= 64 {
            SizeClass::Size64
        } else if size <= 128 {
            SizeClass::Size128
        } else if size <= 256 {
            SizeClass::Size256
        } else if size <= 512 {
            SizeClass::Size512
        } else {
            SizeClass::PageSize
        }
    }

    /// Allocates memory in this size class
    pub fn allocate(self, size: usize) -> *mut u8 {
        match self {
            SizeClass::Size0 => ZST_ADDRESS.as_mut_ptr::<u8>(),
            SizeClass::Size64 => fixed::FIXED_64.allocate(),
            SizeClass::Size128 => fixed::FIXED_128.allocate(),
            SizeClass::Size256 => fixed::FIXED_256.allocate(),
            SizeClass::Size512 => fixed::FIXED_512.allocate(),
            SizeClass::PageSize => paged::allocate((size + 4095) >> 12),
        }
    }

    /// Frees memory in this size class
    pub fn free(self, ptr: *mut u8, size: usize) {
        match self {
            SizeClass::Size0 => {}
            SizeClass::Size64 => fixed::FIXED_64.free(ptr),
            SizeClass::Size128 => fixed::FIXED_128.free(ptr),
            SizeClass::Size256 => fixed::FIXED_256.free(ptr),
            SizeClass::Size512 => fixed::FIXED_512.free(ptr),
            SizeClass::PageSize => paged::free(ptr, (size + 4095) >> 12),
        }
    }
}
