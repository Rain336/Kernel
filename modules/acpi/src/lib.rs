// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Advanced Configuration and Power Interface (ACPI) Module
//!
//! ACPI is a standard developed originally by Intel, now by the UEFI Forum, to enumerate and configure devices installed in a system.
//! The microdragon kernel tries to use as little of it as possible, since most of it should be handled by userspace services.
//! In addition it's a very big standard even including a custom programming language called AML.
//! This Module just allows finding so-called ACPI Tables based on their unique signature,
//! but only until the userspace ACPI service takes over.
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
use common::addr::VirtAddr;
use common::memory::physical_to_virtual;
use common::sync::SyncOnceCell;
use core::mem;
use core::slice;
use interface::ModuleInterface;
use log::{debug, info, log_enabled, warn, Level};

/// Signature of the Extended System Descriptor Table used in ACPI 2.0+.
/// It uses 64-bit pointers, so we have to detect that and choose accordingly.
const EXTENDED_SIGNATURE: &[u8] = b"XSDT";

/// Static reference to the (Extended) System Descriptor Table.
/// This is used by [`find_table`] to iterate though all available ACPI tables.
static SYSTEM_DESCRIPTOR_TABLE: SyncOnceCell<&'static AcpiTableHeader> = SyncOnceCell::new();

/// Entrypoint to the ACPI module.
pub fn init(iface: &ModuleInterface) {
    if SYSTEM_DESCRIPTOR_TABLE.is_initialized() {
        warn!("ACPI Kernel Module already initialized");
        return;
    }

    if iface.rsdp_address == 0 {
        info!("ACPI not available");
        return;
    }
    let address = PhysAddr::new_truncate(iface.rsdp_address);

    // Safety: We assume the address given by the interface is valid.
    let Some(address) = rsdp::read(physical_to_virtual(address)) else {
        return;
    };

    // Safety: We assume the address given by the RSDP is valid.
    let sdp = unsafe { &*physical_to_virtual(address).as_ptr::<AcpiTableHeader>() };
    if !sdp.validate() {
        warn!("ACPI System Descriptor Table corrupted? Checksum didn't match");
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

    SYSTEM_DESCRIPTOR_TABLE.get_or_init(|| sdp);
}

/// Tries to find the ACPI Table `T`.
/// This function only works if [`init`] was called before, else it will always return `None`.
pub fn find_table<T: AcpiTable>() -> Option<&'static T> {
    let sdt = *SYSTEM_DESCRIPTOR_TABLE.get()?;
    let start = VirtAddr::from(sdt as *const _) + mem::size_of::<AcpiTableHeader>();
    let length = sdt.length as usize - mem::size_of::<AcpiTableHeader>();
    debug!("SDP Entry List Start: {:#x} Size: {}", start, length);

    let table = if sdt.signature == EXTENDED_SIGNATURE {
        let count = length / mem::size_of::<u64>();
        let tables = unsafe { slice::from_raw_parts(start.as_ptr::<u64>(), count) };

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
        let tables = unsafe { slice::from_raw_parts(start.as_ptr::<u32>(), count) };

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
