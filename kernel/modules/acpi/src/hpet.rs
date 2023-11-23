// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::{AcpiTable, AcpiTableHeader, AddressStructure};

/// The ACPI table entry describing where the HPET is located.
#[repr(C, packed)]
pub struct HpetTable {
    /// ACPI Table Header
    pub header: AcpiTableHeader,
    pub event_timer_block: u32,
    pub address: AddressStructure,
    pub hpet_number: u8,
    pub minimum_tick: u16,
    pub page_protection: u8,
}

impl AcpiTable for HpetTable {
    const SIGNATURE: &'static [u8; 4] = b"HPET";

    fn header(&self) -> &AcpiTableHeader {
        &self.header
    }
}
