// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::addr::PhysAddr;
use common::memory::physical_to_virtual;
use common::sync::SyncOnceCell;
use core::sync::atomic::{AtomicU64, Ordering};
use log::{debug, trace, warn};

const ENTRY_TYPE_SHIFT: usize = 64 - 8;
const ENTRY_ADDRESS_MASK: u64 = !((u8::MAX as u64) << ENTRY_TYPE_SHIFT);

static INSTANCE: SyncOnceCell<&'static [BitmapEntry]> = SyncOnceCell::new();

pub fn is_initialized() -> bool {
    INSTANCE.is_initialized()
}

pub fn init(map: &'static [BitmapEntry]) -> bool {
    INSTANCE.set(map).is_ok()
}

pub fn allocate() -> PhysAddr {
    let Some(map) = INSTANCE.get() else {
        warn!("Allocate called before Bitmap was setup.");
        return PhysAddr::zero();
    };

    for entry in map.iter() {
        let result = entry.try_allocate();
        if !result.is_null() {
            trace!("Frame allocated at {:p}", result);
            return result;
        }
    }

    warn!("Bitmap exceeded, returning null.");
    PhysAddr::zero()
}

pub struct BitmapEntry {
    level_and_address: u64,
    bitmap: AtomicU64,
}

impl BitmapEntry {
    pub const fn level(&self) -> u8 {
        (self.level_and_address >> ENTRY_TYPE_SHIFT) as u8
    }

    pub const fn address(&self) -> PhysAddr {
        unsafe { PhysAddr::new_unsafe(self.level_and_address & ENTRY_ADDRESS_MASK) }
    }

    pub fn try_allocate(&self) -> PhysAddr {
        try_allocate(self.level(), &self.bitmap, self.address())
    }

    pub fn free(&self, target: PhysAddr) {
        let mut address = self.address();
        let mut offset = (target - address).as_u64();

        for level in (1..=self.level()).rev() {
            let size = size_for_level(level);

            let mask = size - 1;
            let idx = offset & !mask;

            address += idx * size + 4096;
            offset &= mask;
        }
    }
}

fn try_allocate(level: u8, bitmap: &AtomicU64, address: PhysAddr) -> PhysAddr {
    let mut current = bitmap.load(Ordering::Relaxed);
    loop {
        let idx = current.trailing_ones();
        if idx == u64::BITS {
            debug!("Cache miss for Bitmap {:p} at level {}", address, level);
            return PhysAddr::zero();
        }

        if level == 0 {
            let mask = 1 << idx;
            let result = bitmap.fetch_or(mask, Ordering::AcqRel);
            if (result & mask) == 0 {
                return address + idx as u64 * 4096;
            } else {
                current = result;
            }
        } else {
            let address = address + idx as u64 * size_for_level(level);
            let child = unsafe { &*physical_to_virtual(address).as_ptr::<AtomicU64>() };

            let result = try_allocate(level - 1, child, address + 4096u64);

            if result.is_null() {
                let mask = 1 << idx;
                current = bitmap.fetch_or(mask, Ordering::AcqRel) | mask;
            } else {
                return result;
            }
        }
    }
}

const fn size_for_level(level: u8) -> u64 {
    level as u64 * 64 * 4096
}
