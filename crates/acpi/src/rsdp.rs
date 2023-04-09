use common::addr::PhysAddr;
use common::addr::VirtAddr;
use core::mem;
use core::slice;
use log::warn;

/// Structure contianing the pointer to the Root System Descriptor Table.
struct RootSystemDescriptionPointer {
    signature: [u8; 8],
    checksum: u8,
    oedm_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

impl RootSystemDescriptionPointer {
    /// Validates the structure, returns true if vaild.
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

/// Structure contianing the pointer to the Extended System Descriptor Table.
/// Used starting ACPI 2.0+.
struct RootSystemDescriptionPointer2 {
    base: RootSystemDescriptionPointer,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

impl RootSystemDescriptionPointer2 {
    /// Validates the structure, returns true if vaild.
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
