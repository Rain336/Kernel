use core::{mem, slice};
use log::debug;
use uefi::table::cfg::{ConfigTableEntry, ACPI2_GUID, ACPI_GUID};

#[repr(C, packed)]
struct RootSystemDescriptionPointer {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RootSystemDescriptionPointer {
    pub fn validate(&self) -> bool {
        let bytes = unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<RootSystemDescriptionPointer>(),
            )
        };
        let checksum: u8 = bytes.iter().fold(0, |l, r| l.wrapping_add(*r));

        debug!(
            "ACPI 1, OEM: {}, Revision: {}",
            core::str::from_utf8(&self.oem_id).unwrap_or("Unknown"),
            self.revision
        );

        &self.signature == b"RSD PTR " && checksum == 0
    }
}

#[repr(C, packed)]
struct ExtendedRootSystemDescriptionPointer {
    base: RootSystemDescriptionPointer,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

impl ExtendedRootSystemDescriptionPointer {
    pub fn validate(&self) -> bool {
        let bytes = unsafe {
            slice::from_raw_parts(
                self as *const _ as *const u8,
                mem::size_of::<ExtendedRootSystemDescriptionPointer>(),
            )
        };
        let checksum: u8 = bytes.iter().fold(0, |l, r| l.wrapping_add(*r));

        debug!(
            "ACPI 2, OEM: {}, Revision: {}",
            core::str::from_utf8(&self.base.oem_id).unwrap_or("Unknown"),
            self.base.revision
        );

        &self.base.signature == b"RSD PTR " && checksum == 0
    }
}

pub fn find_sdt_address(table: &[ConfigTableEntry]) -> u64 {
    if let Some(x) = table.iter().find(|x| x.guid == ACPI2_GUID) {
        let xsdp = unsafe { &*(x.address as *const ExtendedRootSystemDescriptionPointer) };
        if xsdp.validate() {
            xsdp.xsdt_address
        } else {
            0
        }
    } else if let Some(x) = table.iter().find(|x| x.guid == ACPI_GUID) {
        let rsdp = unsafe { &*(x.address as *const RootSystemDescriptionPointer) };
        if rsdp.validate() {
            rsdp.rsdt_address as u64
        } else {
            0
        }
    } else {
        0
    }
}
