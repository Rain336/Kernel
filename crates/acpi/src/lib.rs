//! # Advanced Configuration and Power Interface (ACPI)
//!
//! ACPI is a standard developed originally by Intel, now by the UEFI Forum, to enumerate and configure devices installed in a system.
//! The microdragon kernel tries to use as little of it as posible, since most of it should be handled by userspace services.
//! In adition it's a very big standard even including a custom programming language called AML.
//!
#![no_std]

mod header;
mod hpet;
mod rsdp;
mod table;
mod utils;

pub use header::*;
pub use hpet::*;
pub use table::*;
pub use utils::*;

use common::addr::PhysAddr;
use common::memory::physical_to_virtual;
use common::sync::SyncOnceCell;
use core::mem;
use core::slice;
use limine::LimineRsdpRequest;
use log::{debug, info, log_enabled, warn, Level};

/// Signature of the Extended System Descriptor Table used in ACPI 2.0+.
/// It uses 64-bit pointers, so we have to detect that and choose acordingly.
const EXTENDED_SIGNATURE: &[u8] = b"XSDT";

/// Request to the bootloader to give us a pointer to the Root System Description Pointer.
static RSDP_REQUEST: LimineRsdpRequest = LimineRsdpRequest::new(0);

/// Static referance to the (Extended) System Descriptor Table.
/// This is used by [`find_table`] to iterate though all available ACPI tables.
static SYSTEM_DESCRIPTION_TABLE: SyncOnceCell<&'static AcpiTableHeader> = SyncOnceCell::new();

/// Initializes the crate, by requesting the root system description pointer from the bootloader.
/// This is needed for [`find_table`] to function.
pub fn init() {
    let Some(response) = RSDP_REQUEST.get_response().get() else {
        info!("ACPI not available");
        return;
    };
    let Some(address) = response.address.as_ptr() else {
        warn!("Bootloader supplied null pointer to RSDP");
        return;
    };
    let address = PhysAddr::new_truncate(address as u64);

    let Some(address) = rsdp::read(physical_to_virtual(address)) else {
        return;
    };

    let sdp = unsafe { &*physical_to_virtual(address).as_ptr::<AcpiTableHeader>() };
    if !sdp.validate() {
        warn!("ACPI System Description Table corrupted? Checksum didn't match");
        return;
    }

    if log_enabled!(Level::Debug) {
        let oem_revision = sdp.oem_revision;
        let creator_revision = sdp.creator_revision;
        debug!(
            "ACPI {} Revision: {} OEM: {} OEM Table: {} OEM Revision: {} AML Compiler: {} AML Compiler Revision: {}",
            core::str::from_utf8(&sdp.signature).unwrap_or_default().trim(),
            sdp.revision,
            core::str::from_utf8(&sdp.oem_id).unwrap_or_default().trim(),
            core::str::from_utf8(&sdp.oem_table_id).unwrap_or_default().trim(),
            oem_revision,
            core::str::from_utf8(&sdp.creator_id).unwrap_or_default().trim(),
            creator_revision
        );
    }

    SYSTEM_DESCRIPTION_TABLE.get_or_init(|| sdp);
}

/// Tries to find the ACPI Table `T`.
/// This function only works if [`init`] was called before, else it will always return `None`.
pub fn find_table<T: AcpiTable>() -> Option<&'static T> {
    let sdp = SYSTEM_DESCRIPTION_TABLE.get()?;
    let start = ((*sdp) as *const _ as usize) + mem::size_of::<AcpiTableHeader>();
    let length = sdp.length as usize - mem::size_of::<AcpiTableHeader>();
    debug!("SDP Entry List Start: {:#x} Size: {}", start, length);

    let table = if sdp.signature == EXTENDED_SIGNATURE {
        let count = length / mem::size_of::<u64>();
        let tables = unsafe { slice::from_raw_parts(start as *const u64, count) };

        tables.iter().find_map(|x| {
            let table = PhysAddr::new_truncate(*x);
            let table = unsafe { &*physical_to_virtual(table).as_ptr::<AcpiTableHeader>() };

            if &table.signature == T::SIGNATURE {
                Some(table)
            } else {
                None
            }
        })?
    } else {
        let count = length / mem::size_of::<u32>();
        let tables = unsafe { slice::from_raw_parts(start as *const u32, count) };

        tables.iter().find_map(|x| {
            let table = PhysAddr::new_truncate(*x as u64);
            let table = unsafe { &*physical_to_virtual(table).as_ptr::<AcpiTableHeader>() };

            if &table.signature == T::SIGNATURE {
                Some(table)
            } else {
                None
            }
        })?
    };

    debug!(
        "Found ACPI Table '{}'",
        core::str::from_utf8(&table.signature).unwrap_or_default()
    );

    if !table.validate() {
        warn!(
            "ACPI Table '{}' corrupted? Checksum didn't match",
            core::str::from_utf8(&table.signature).unwrap_or_default()
        );
        return None;
    }

    Some(unsafe { &*(table as *const _ as *const T) })
}
