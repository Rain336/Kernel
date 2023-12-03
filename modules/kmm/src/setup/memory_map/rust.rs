// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Rust Bootloader Memory Map
//!
//! Provides the read in routine for Rust Bootloader format memory maps.
//!
use bootloader_api::info::{MemoryRegion, MemoryRegionKind};
use core::ptr::NonNull;
use core::{ptr, slice};
use interface::MemoryMapInfo;
use log::{error, trace, warn};

use crate::pmm::FreeListEntry;

/// Reads in the memory map from the boot loader, passes the free regions to the PMM and returns the highest physical memory address.
pub fn read_rust_memory_map(map: &MemoryMapInfo) -> u64 {
    // SAFETY: We can assume that reads from map.memory_map to map.memory_map_size are valid and the alignment is asserted.
    let slice = unsafe {
        slice::from_raw_parts(map.memory_map as *const MemoryRegion, map.memory_map_count)
    };

    let mut list = None;
    let mut idx = 0;
    let mut max = 0;
    for region in slice {
        trace!(
            "Memory Region - Start: {:x} End: {} Kind: {}",
            region.start,
            region.end,
            match region.kind {
                MemoryRegionKind::Usable => 100,
                MemoryRegionKind::Bootloader => 101,
                MemoryRegionKind::UnknownUefi(x) => x,
                MemoryRegionKind::UnknownBios(x) => x,
                _ => 404,
            }
        );

        if region.end > max {
            max = region.end;
        }

        if region.end <= super::RESERVED_AREA {
            continue;
        }

        match region.kind {
            MemoryRegionKind::Usable
            | MemoryRegionKind::Bootloader
            | MemoryRegionKind::UnknownBios(1)
            | MemoryRegionKind::UnknownUefi(1) // Loader Code
            | MemoryRegionKind::UnknownUefi(2) // Loader Data
            | MemoryRegionKind::UnknownUefi(3) // Boot Services Code
            | MemoryRegionKind::UnknownUefi(4) // Boot Services Data
            | MemoryRegionKind::UnknownUefi(7) => { // Usable
                let len = region.end - region.start;

                if len < 4096 {
                    warn!(
                        "Region too small for mapping at {:x} with {} bytes",
                        region.start, len
                    );
                    continue;
                }

                let start = if region.start < super::RESERVED_AREA {
                    super::RESERVED_AREA
                } else {
                    region.start
                };

                match &mut list {
                    None => {
                        // Safety: The memory is marked as available and can be written to. We checked beforehand that the region is at least 4096 bytes long.
                        unsafe { ptr::write_bytes(start as *mut u8, 0, 4096) };

                        // Safety: Same as above, plus the zero installation makes this a valid representation.
                        let slice =
                            unsafe { slice::from_raw_parts_mut(start as *mut FreeListEntry, 256) };

                        if (region.end - start) >= 8192 {
                            slice[0] = FreeListEntry {
                                start: start + 4096,
                                end: region.end,
                            };

                            idx += 1;
                        }

                        list = Some(slice);
                    }
                    Some(slice) => {
                        slice[idx] = FreeListEntry { start, end: region.end };

                        idx += 1;
                    }
                }
            },
            MemoryRegionKind::UnknownUefi(ty) => match ty {
                0 | 5 | 6 | 11 | 12 | 13 => {
                    // Reserved memory | Runtime Services Code | Runtime Services Data | MMIO | MMIO Port Space | PAL Code
                },
                8 => error!(
                    "Bad memory detected! In region {:x} - {:x}",
                    region.start, region.end
                ),
                9 => {
                    // ACPI reclaimable
                },
                10 | 14 => {
                    // ACPI NVS memory | Persistent Memory
                }
                _ => warn!("Unknown UEFI memory type: {}", ty),
            },
            MemoryRegionKind::UnknownBios(ty) => match ty {
                2 => {
                    // Reserved memory. Do not touch.
                }
                3 => {
                    // ACPI reclaimable
                }
                4 => {
                    // ACPI NVS memory
                }
                5 => error!(
                    "Bad memory detected! In region {:x} - {:x}",
                    region.start, region.end
                ),
                _ => warn!("Unknown BIOS memory type: {}", ty),
            },
            _ => {}
        }
    }

    let list = list.expect("No usable memory in memory map");
    //list.sort_unstable_by_key(|x| x.address);

    crate::pmm::init(NonNull::new(list.as_mut_ptr()).unwrap(), list.len());

    trace!("Highest physical address: {:x}", max);

    max
}
