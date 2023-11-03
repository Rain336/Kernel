/// An ACPI address structure struct used in some tables.
#[repr(C, packed)]
pub struct AddressStructure {
    pub address_space_id: u8,
    pub register_bit_width: u8,
    pub register_bit_offset: u8,
    pub reserved: u8,
    pub address: u64,
}
