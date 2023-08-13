//! # Free Tracker Module
//!
//! The FreeTracker is a b-tree map containing blocks of memory.
//! The key is the starting address, with the value being the ending address.
use alloc::collections::BTreeMap;
use common::addr::PhysAddr;
use core::cmp::Ordering;
use spinning_top::Spinlock;

static FREE_TRACKER: Spinlock<BTreeMap<PhysAddr, PhysAddr>> = Spinlock::new(BTreeMap::new());

/// Tries to allocate a single 4KiB page from the FreeTracker, if possible.
/// Returns `None` when the free tracker is out of memory
pub fn allocate_page() -> Option<PhysAddr> {
    let mut guard = FREE_TRACKER.lock();
    let (start, end) = guard.pop_first()?;

    let bytes = (end - start).as_u64();
    if bytes > 4096 {
        guard.insert(start + 4096u64, end);
    }

    Some(start)
}

/// Allocates a the given number of bytes of memory.
/// The allocation might be split up into multiple regions, so the given function is called for each region.
/// If all bytes could be allocated, the function returns `true`.
pub fn allocate(bytes: u64, mut f: impl FnMut(PhysAddr, u64)) -> bool {
    let mut guard = FREE_TRACKER.lock();
    loop {
        let (start, end) = match guard.pop_first() {
            Some(x) => x,
            None => return false,
        };
        let size = (end - start).as_u64();

        match bytes.cmp(&size) {
            Ordering::Less => {
                guard.insert(start + bytes, end);
                f(start, bytes);
                break true;
            }
            Ordering::Equal => {
                f(start, bytes);
                break true;
            }
            Ordering::Greater => {
                f(start, size);
            }
        }
    }
}

/// Marks the given block of memory as free.
pub fn free(start: PhysAddr, bytes: u64) {
    let mut guard = FREE_TRACKER.lock();
    let end = start + bytes;

    if let Some(end) = guard.remove(&end) {
        guard.insert(start, end);
    } else {
        guard.insert(start, end);
    }
}
