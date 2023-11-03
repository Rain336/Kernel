//! # Physical Memory Manager (PMM) Module
//!
//! The physical memory manager (PMM) is responsible for keeping track of free physical memory frames.
//! The current implementation uses a list of free memory regions behind a mutex.
//!
use common::addr::PhysAddr;
use common::sync::{CriticalSection, SyncOnceCell};
use core::ptr::NonNull;
use core::slice;
use log::error;
use memory::frame::PhysFrame;
use spinning_top::Spinlock;

/// Global instance of the [`FreeList`] management struct.
static INSTANCE: SyncOnceCell<Spinlock<FreeList>> = SyncOnceCell::new();

/// An entry in the free list.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FreeListEntry {
    /// Starting address of the free region.
    pub start: u64,

    /// Ending address of the free region.
    pub end: u64,
}

/// Management struct of the free list.
struct FreeList {
    /// Pointer to the free list array.
    ptr: NonNull<FreeListEntry>,

    /// Number of initialized elements in the array.
    size: usize,
}

impl FreeList {
    /// Returns the free list as a mutable slice.
    fn as_slice_mut(&mut self) -> &mut [FreeListEntry] {
        // SAFETY: ptr is valid for self.size items and owned by FreeList.
        unsafe { slice::from_raw_parts_mut(self.ptr.as_mut(), self.size) }
    }

    /// Removes the entry at the given index by shifting all lower entries up and deceasing the size.
    fn compact(&mut self, idx: usize) {
        let shift = idx + 1;

        if shift != self.size {
            self.as_slice_mut().copy_within((idx + 1).., idx);
        }

        self.size -= 1;
    }

    /// Appends a new entry to the end of the list.
    fn append(&mut self, start: u64, end: u64) {
        if self.size == 256 {
            error!("FreeList exceeded");
            return;
        }

        let idx = self.size;
        self.size += 1;
        self.as_slice_mut()[idx] = FreeListEntry { start, end };
    }
}

unsafe impl Send for FreeList {}
unsafe impl Sync for FreeList {}

/// Initializes the Physical Memory Manager.
pub fn init(ptr: NonNull<FreeListEntry>, size: usize) {
    assert!(
        INSTANCE.set(Spinlock::new(FreeList { ptr, size })).is_ok(),
        "PMM init called twice"
    );
}

/// Returns whenever the Physical Memory Manager is already initialized.
pub fn is_initialized() -> bool {
    INSTANCE.is_initialized()
}

/// Allocates a singe physical frame.
/// Returns `null` if the allocation fails.
pub fn allocate() -> PhysFrame {
    let _section = CriticalSection::new();
    let Some(mut guard) = INSTANCE.get().map(|x| x.lock()) else {
        return PhysFrame::containing_address(PhysAddr::zero());
    };

    let mut address = PhysAddr::zero();
    let mut compact = None;
    for (i, entry) in guard.as_slice_mut().iter_mut().enumerate() {
        let size = entry.end - entry.start;

        if size < 4096 {
            continue;
        }

        address = PhysAddr::new_truncate(entry.start);
        entry.start += 4096;

        if size == 4096 {
            compact = Some(i);
        }

        break;
    }

    if let Some(idx) = compact {
        guard.compact(idx);
    }

    PhysFrame::containing_address(address)
}

/// Allocates a block of continues physical frame.
/// Returns `null` if the allocation fails.
pub fn allocate_block(frames: usize) -> PhysAddr {
    let bytes = frames as u64 * 4096;

    let _section = CriticalSection::new();
    let Some(mut guard) = INSTANCE.get().map(|x| x.lock()) else {
        return PhysAddr::zero();
    };

    let mut address = PhysAddr::zero();
    let mut compact = None;
    for (i, entry) in guard.as_slice_mut().iter_mut().enumerate() {
        let size = entry.end - entry.start;

        if size < bytes {
            continue;
        }

        address = PhysAddr::new_truncate(entry.start);
        entry.start += bytes;

        if size == bytes {
            compact = Some(i);
        }

        break;
    }

    if let Some(idx) = compact {
        guard.compact(idx);
    }

    address
}

/// Frees a singe physical frame previously allocated by [`allocate`]
pub fn free(frame: PhysFrame) {
    let start = frame.start_address().as_u64();
    let end = start + 4096;

    let _section = CriticalSection::new();
    let Some(mut guard) = INSTANCE.get().map(|x| x.lock()) else {
        return;
    };

    for entry in guard.as_slice_mut() {
        if entry.start == end {
            entry.start = start;
            return;
        } else if entry.end == start {
            entry.end = end;
            return;
        }
    }

    guard.append(start, end);
}

/// Frees a block of continues physical frames previously allocated by [`allocate_block`].
pub fn free_block(address: PhysAddr, frames: usize) {
    let bytes = frames as u64 * 4096;
    let start = address.as_u64();
    let end = start + bytes;

    let _section = CriticalSection::new();
    let Some(mut guard) = INSTANCE.get().map(|x| x.lock()) else {
        return;
    };

    for entry in guard.as_slice_mut() {
        if entry.start == end {
            entry.start = start;
            return;
        } else if entry.end == start {
            entry.end = end;
            return;
        }
    }

    guard.append(start, end);
}
