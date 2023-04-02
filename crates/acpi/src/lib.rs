//! # Advanced Configuration and Power Interface (ACPI)
//!
//! This crate implements the ACPI Driver of the microdragon kernel.
//! Microdragon tries to use as little ACPI as possible, since most of it should be handled by userspace services.
#![no_std]

mod header;
mod hpet;
mod table;
mod utils;

pub use header::*;
pub use hpet::*;
pub use table::*;
pub use utils::*;

use common::sync::SyncOnceCell;
use core::mem;
use core::ptr;
use log::log_enabled;
use log::Level;
use log::{debug, warn};

const EXTENDED_SIGNATURE: &[u8] = b"XSDT";

static SYSTEM_DESCRIPTION_TABLE: SyncOnceCell<&'static AcpiTableHeader> = SyncOnceCell::new();

/// Initializes the crate, by suppling it the address of the ACPI (Extended) System Descriptor Table.
/// This is needed for [`find_table`] to function.
pub fn init(sdp: u64) {
    if sdp == 0 {
        // We should panic in this case, I think...
        return;
    }

    let sdp = unsafe { &*(sdp as *const AcpiTableHeader) };
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

    if sdp.signature == EXTENDED_SIGNATURE {
        let count = length / mem::size_of::<u64>();

        for i in 0..count {
            let table =
                unsafe { ptr::read_unaligned((start + i * mem::size_of::<u64>()) as *const u64) };
            let table = unsafe { &*(table as *const AcpiTableHeader) };

            if &table.signature == T::SIGNATURE && table.validate() {
                debug!(
                    "Found table {}",
                    core::str::from_utf8(&table.signature).unwrap_or_default()
                );
                return Some(unsafe { &*(table as *const _ as *const T) });
            }
        }

        None
    } else {
        let count = length / mem::size_of::<u32>();

        for i in 0..count {
            let table =
                unsafe { ptr::read_unaligned((start + i * mem::size_of::<u32>()) as *const u32) };
            let table = unsafe { &*(table as *const AcpiTableHeader) };

            if &table.signature == T::SIGNATURE && table.validate() {
                debug!(
                    "Found table {}",
                    core::str::from_utf8(&table.signature).unwrap_or_default()
                );
                return Some(unsafe { &*(table as *const _ as *const T) });
            }
        }

        None
    }
}
