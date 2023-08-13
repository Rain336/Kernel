use alloc::collections::BTreeMap;
use common::addr::PhysAddr;
use limine::{LimineMemmapRequest, LimineMemoryMapEntryType};
use spinning_top::Spinlock;

static MEMORY_MAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(1);

pub struct MemoryMapEntry {
    end: PhysAddr,
    ty: LimineMemoryMapEntryType,
}

static MEMORY_MAP: Spinlock<BTreeMap<PhysAddr, MemoryMapEntry>> = Spinlock::new(BTreeMap::new());

pub fn read_memory_map() {
    let response = MEMORY_MAP_REQUEST
        .get_response()
        .get()
        .expect("No memory map supplied by bootloader");

    let mut guard = MEMORY_MAP.lock();

    for entry in response.memmap() {
        let start = PhysAddr::new_truncate(entry.base);

        guard.insert(
            start,
            MemoryMapEntry {
                end: start + entry.len,
                ty: entry.typ,
            },
        );

        if matches!(
            entry.typ,
            LimineMemoryMapEntryType::Usable | LimineMemoryMapEntryType::BootloaderReclaimable
        ) {
            if let Some(start) = start.align_up(4096) {
                crate::free::free(start, entry.len & (!4096));
            }
        }
    }
}
