// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use core::slice;

/// The header of every ACPI table.
#[repr(C, packed)]
pub struct AcpiTableHeader {
    /// 4 byte signature of this table.
    pub signature: [u8; 4],

    /// Length of the whole table in bytes.
    pub length: u32,

    /// Revision of this table implemented.
    pub revision: u8,

    /// Checksum for validating.
    pub checksum: u8,

    /// OEM Id
    pub oem_id: [u8; 6],

    /// OEM Table Id
    pub oem_table_id: [u8; 8],

    /// OEM Revision
    pub oem_revision: u32,

    /// AML Compiler Id
    pub creator_id: [u8; 4],

    /// AML Compiler Revision
    pub creator_revision: u32,
}

impl AcpiTableHeader {
    /// Validates the ACPI table against it's checksum.
    pub fn validate(&self) -> bool {
        let bytes =
            unsafe { slice::from_raw_parts(self as *const _ as *const u8, self.length as usize) };
        let checksum = bytes.iter().fold(0u8, |l, r| l.wrapping_add(*r));

        checksum == 0
    }
}
