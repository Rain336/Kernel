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
