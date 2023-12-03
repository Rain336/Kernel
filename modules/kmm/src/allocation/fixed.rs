// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Fixed Size Allocators
//!
//! This module contains a fixed size allocator with four size classes 64, 128, 256 and 512 bytes.
//! Each size class has it's own chain of pages that that lazily adds new pages onto itself when needed.
//! The page is segmented into fixed size blocks based on the size class.
//! The blocks are tracked using an atomic bitmap.
//!
use alloc::boxed::Box;
use common::addr::VirtAddr;
use common::interrupts;
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};

/// Marker value to notify that a new page is being allocated.
const UPDATING: usize = usize::MAX;

/// Bookkeeping struct for one page of blocks.
#[repr(C)]
pub struct FixedAllocator<T> {
    /// Pointer to the page.
    page: *mut u8,

    /// Bitmap to keep track of which blocks are in use.
    bitmap: T,

    /// Pointer to the next page's bookkeeping struct.
    next: AtomicPtr<FixedAllocator<T>>,
}

impl<T> FixedAllocator<T> {
    /// Tries to load the bookkeeping struct of the next page in the chain.
    fn try_get_next(&self) -> Option<&FixedAllocator<T>> {
        // First we try to load the next pointer relaxed to see if we already get a result.
        let mut next = self.next.load(Ordering::Relaxed);

        if next.is_null() || next as usize == UPDATING {
            // if the next pointer is null or updating...
            next = self.next.load(Ordering::Acquire); // ...we try to load it again with a stronger memory ordering.

            if next.is_null() {
                // If the pointer is still null, we return None, since we are the end of the list.
                return None;
            }

            while next as usize == UPDATING {
                // If the pointer is updating, we loop and try to load a new value until it's set.
                next = self.next.load(Ordering::Acquire);
            }
        }

        // If the relaxed load worked or the bookkeeping struct finished initializing, we return next as a valid reference.
        // SAFETY: next is neither null nor UPDATING, so we can assume a valid pointer to a FixedAllocator<T> struct.
        Some(unsafe { &*next })
    }
}

unsafe impl<T: Send> Send for FixedAllocator<T> {}
unsafe impl<T: Send> Sync for FixedAllocator<T> {}

macro_rules! fixed_allocator {
    ($atomic:ty, $size:literal, $bits:literal) => {
        impl FixedAllocator<$atomic> {
            /// Creates a bookkeeping struct for the given page.
            /// The page is assumed to be 4KiB in size.
            pub const fn new(page: *mut u8) -> Self {
                FixedAllocator {
                    page,
                    bitmap: <$atomic>::new(0),
                    next: AtomicPtr::new(ptr::null_mut()),
                }
            }

            /// Allocates a block from this page or asks the next page to allocate, if full.
            pub fn allocate(&self) -> *mut u8 {
                // We load the tracking bitmap relaxed, since we are going to compare_exchange later in the loop, this is no problem.
                let mut current = self.bitmap.load(Ordering::Relaxed);

                loop {
                    let offset = current.trailing_ones(); // We try to find a free block (0 bit in the bitmap).
                    if offset == $bits {
                        // If we couldn't find one, we get the next page and try to allocate there.
                        return self.get_or_create_next().allocate();
                    }

                    let bit = 1 << offset; // We set the bit in the bitmap and
                    current = self.bitmap.fetch_or(bit, Ordering::AcqRel);

                    // if the bit wasn't already set, we got the allocation and return the block.
                    if (current & bit) == 0 {
                        // SAFETY: self.page is valid for up to 4096 bytes and none of the defined fixed allocators exceed that.
                        return unsafe { self.page.add(offset as usize * $size) };
                    }
                }
            }

            /// Frees a block from this page or asks the next page if the pointer is not from this one.
            /// Returns true if the block could be deallocated.
            pub fn free(&self, pointer: *const u8) {
                let ptr = pointer as usize;
                let page = self.page as usize;

                if ptr < page || ptr > page.saturating_add(4096) {
                    // If the pointer is not inside the range of our page we try to get the next bookkeeping struct in the chain.
                    if let Some(next) = self.try_get_next() {
                        // If succeeded, we call free on it.
                        next.free(pointer)
                    } else {
                        // Otherwise, if we are at the end of the chain, we log that the pointer wasn't allocated with us.
                        log::warn!("Pointer {:p} not allocated with this Allocator", pointer);
                    }
                } else {
                    // If the pointer is inside our page, we cerate a and mask to unset the bit in the bitmap
                    let offset = (ptr - page) / $size;
                    let mask = !(1 << offset);
                    self.bitmap.fetch_and(mask, Ordering::AcqRel);
                }
            }

            /// Returns the bookkeeping struct of the next page or allocates a new page with bookkeeping struct.
            fn get_or_create_next(&self) -> &FixedAllocator<$atomic> {
                let guard = interrupts::disable(); // This function can deadlock, so we start a critical section.

                match self.next.compare_exchange(
                    ptr::null_mut(),
                    UPDATING as *mut FixedAllocator<$atomic>,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    // We try to set the next pointer to UPDATING (lock the pointer)
                    Ok(_) => {
                        // If we managed to lock it, we have to allocate a page.
                        let page = allocate_page().as_mut_ptr::<u8>();

                        // Next we have to allocate the bookkeeping struct with a bit of a twist.
                        let fixed = if $bits == 64 {
                            // If we are the 64 bytes size class, we put the bookkeeping struct into the first block.
                            let fixed = page as *mut FixedAllocator<$atomic>;

                            // SAFETY: The Layout of FixedAllocator fits into once memory page.
                            unsafe {
                                *fixed = FixedAllocator {
                                    page,
                                    bitmap: <$atomic>::new(1),
                                    next: AtomicPtr::new(ptr::null_mut()),
                                };
                            }

                            fixed
                        } else {
                            // In all other size classes we just allocate the struct using Box and GlobalAlloc.
                            Box::into_raw(Box::new(FixedAllocator {
                                page,
                                bitmap: <$atomic>::new(0),
                                next: AtomicPtr::new(ptr::null_mut()),
                            }))
                        };

                        // Finally we store the bookkeeping struct in the next pointer, unlocking it at the same time.
                        self.next.store(fixed, Ordering::Release);

                        // SAFETY: fixed is either allocated using Box or initialized by a write.
                        unsafe { &*fixed }
                    }
                    Err(mut current) => {
                        // If we didn't manage to set the pointer to UPDATING (someone else got the lock),
                        // we first drop the critical section, since we can't deadlock anymore and might be about to spin loop
                        drop(guard);

                        // Now we spin loop and check if the lock has been released.
                        while current as usize == UPDATING {
                            current = self.next.load(Ordering::Acquire);
                        }

                        // Finally we return the bookkeeping struct that got created.
                        // SAFETY: next is neither null nor UPDATING, so we can assume a valid pointer to a FixedAllocator<T> struct.
                        unsafe { &*current }
                    }
                }
            }
        }
    };
}

fixed_allocator!(AtomicU64, 64, 64);
fixed_allocator!(AtomicU32, 128, 32);
fixed_allocator!(AtomicU16, 256, 16);
fixed_allocator!(AtomicU8, 512, 8);

static mut FIXED_64_BLOCK: [u8; 4096] = [0; 4096];
pub static FIXED_64: FixedAllocator<AtomicU64> =
    FixedAllocator::<AtomicU64>::new(unsafe { FIXED_64_BLOCK.as_ptr() as *mut _ });

static mut FIXED_128_BLOCK: [u8; 4096] = [0; 4096];
pub static FIXED_128: FixedAllocator<AtomicU32> =
    FixedAllocator::<AtomicU32>::new(unsafe { FIXED_128_BLOCK.as_ptr() as *mut _ });

static mut FIXED_256_BLOCK: [u8; 4096] = [0; 4096];
pub static FIXED_256: FixedAllocator<AtomicU16> =
    FixedAllocator::<AtomicU16>::new(unsafe { FIXED_256_BLOCK.as_ptr() as *mut _ });

static mut FIXED_512_BLOCK: [u8; 4096] = [0; 4096];
pub static FIXED_512: FixedAllocator<AtomicU8> =
    FixedAllocator::<AtomicU8>::new(unsafe { FIXED_512_BLOCK.as_ptr() as *mut _ });

#[inline]
fn allocate_page() -> VirtAddr {
    // SAFETY: paged::allocate returns a valid pointer, just not as VirtAddr.
    unsafe { VirtAddr::new_unsafe(super::paged::allocate(1) as u64) }
}
