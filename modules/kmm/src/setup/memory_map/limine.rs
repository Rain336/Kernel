// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Limine Memory Map
//!
//! Provides the read in routine for Liminie format memory maps.
//!
use crate::pmm::FreeListEntry;
use core::ptr::{self, NonNull};
use core::slice;
use interface::MemoryMapInfo;
use limine::{MemmapEntry, MemoryMapEntryType};
use log::{error, trace, warn};

/// Reads in the memory map from the boot loader, passes the free regions to the PMM and returns the highest physical memory address.
pub fn read_limine_memory_map(map: &MemoryMapInfo) -> u64 {
    // SAFETY: We can assume that reads from map.memory_map to map.memory_map_size are valid and the alignment is asserted.
    let slice = unsafe {
        slice::from_raw_parts(
            map.memory_map as *const *const MemmapEntry,
            map.memory_map_count,
        )
    };

    let mut list = None;
    let mut idx = 0;
    let mut max = 0;
    for entry in slice {
        let entry = unsafe { &**entry };
        let end = entry.base + entry.len;

        trace!(
            "Memory Map Entry - TY: {} Start: {:x} Size: {}",
            entry.typ as u32,
            entry.base,
            entry.len
        );

        if entry.typ != MemoryMapEntryType::Reserved && end > max {
            max = end;
        }

        if end <= super::RESERVED_AREA {
            continue;
        }

        match entry.typ {
            MemoryMapEntryType::Usable | MemoryMapEntryType::BootloaderReclaimable => {
                if entry.len < 4096 {
                    warn!(
                        "Region too small for mapping at {:x} with {} bytes",
                        entry.base, entry.len
                    );
                    continue;
                }

                let start = if entry.base < super::RESERVED_AREA {
                    super::RESERVED_AREA
                } else {
                    entry.base
                };

                match &mut list {
                    None => {
                        // Safety: The memory is marked as available and can be written to. We checked beforehand that the region is at least 4096 bytes long.
                        unsafe { ptr::write_bytes(start as *mut u8, 0, 4096) };

                        // Safety: Same as above, plus the zero installation makes this a valid representation.
                        let slice =
                            unsafe { slice::from_raw_parts_mut(start as *mut FreeListEntry, 256) };

                        if (end - start) >= 8192 {
                            slice[0] = FreeListEntry {
                                start: start + 4096,
                                end,
                            };

                            idx += 1;
                        }

                        list = Some(slice);
                    }
                    Some(slice) => {
                        slice[idx] = FreeListEntry { start, end };

                        idx += 1;
                    }
                }
            }
            MemoryMapEntryType::Reserved
            | MemoryMapEntryType::KernelAndModules
            | MemoryMapEntryType::Framebuffer => {
                // Do not touch.
            }
            MemoryMapEntryType::AcpiReclaimable => {
                // TODO: Store and free after they have been read in by the userspace ACPI service.
            }
            MemoryMapEntryType::AcpiNvs => {
                // TODO: Tell the userspace ACPI service about these.
            }
            MemoryMapEntryType::BadMemory => {
                // TODO: Maybe remember these so the user can look them up or something...
                error!(
                    "Bad memory detected! At address {:x} for {} bytes",
                    entry.base, entry.len
                );
            }
        }
    }

    let list = list.expect("No usable memory in memory map");
    //list.sort_unstable_by_key(|x| x.address);

    crate::pmm::init(NonNull::new(list.as_mut_ptr()).unwrap(), list.len());

    trace!("Highest physical address: {:x}", max);

    max
}
