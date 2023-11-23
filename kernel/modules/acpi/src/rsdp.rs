// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::addr::PhysAddr;
use common::addr::VirtAddr;
use core::mem;
use core::slice;
use log::warn;

/// Structure containing the pointer to the Root System Descriptor Table.
#[repr(C, packed)]
struct RootSystemDescriptionPointer {
    _signature: [u8; 8],
    _checksum: u8,
    _oedm_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RootSystemDescriptionPointer {
    /// Validates the structure, returns true if valid.
    fn validate(&self) -> bool {
        let bytes = unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<RootSystemDescriptionPointer>(),
            )
        };
        let checksum = bytes.iter().fold(0u8, |l, r| l.wrapping_add(*r));

        checksum == 0
    }
}

/// Structure containing the pointer to the Extended System Descriptor Table.
/// Used starting ACPI 2.0+.
#[repr(C, packed)]
struct RootSystemDescriptionPointer2 {
    _base: RootSystemDescriptionPointer,
    length: u32,
    xsdt_address: u64,
    _extended_checksum: u8,
    _reserved: [u8; 3],
}

impl RootSystemDescriptionPointer2 {
    /// Validates the structure, returns true if valid.
    fn validate(&self) -> bool {
        let bytes =
            unsafe { slice::from_raw_parts(self as *const _ as *const u8, self.length as usize) };
        let checksum = bytes.iter().fold(0u8, |l, r| l.wrapping_add(*r));

        checksum == 0
    }
}

/// Reads the System Descriptor Table from a pointer to the Root System Description Pointer.
/// It checks if it's the ACPI 2 version and returns the Extended System Descriptor Table instead.
pub fn read(address: VirtAddr) -> Option<PhysAddr> {
    let rsdp = unsafe { &*address.as_ptr::<RootSystemDescriptionPointer>() };
    if !rsdp.validate() {
        warn!("ACPI Root System Description Pointer corrupted? Checksum didn't match");
        return None;
    }

    if rsdp.revision == 2 {
        let rsdp = unsafe { &*address.as_ptr::<RootSystemDescriptionPointer2>() };
        if !rsdp.validate() {
            warn!("ACPI Root System Description Pointer 2.0 corrupted? Checksum didn't match");
            return None;
        }

        Some(PhysAddr::new_truncate(rsdp.xsdt_address))
    } else {
        Some(PhysAddr::new_truncate(rsdp.rsdt_address as u64))
    }
}
