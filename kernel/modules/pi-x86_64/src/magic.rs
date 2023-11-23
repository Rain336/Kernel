// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::addr::PhysAddr;
use core::arch::asm;
use x86_64::registers::control::Cr3;

/// Gets the physical address of the root page table.
///
/// Implemented-By: PI
#[export_name = "__internal_get_root_page_table"]
fn get_root_page_table() -> PhysAddr {
    let (frame, _) = Cr3::read_raw();
    // Safety: A valid PhysAddr from the x86_64 crate is also valid for the common crate.
    unsafe { PhysAddr::new_unsafe(frame.start_address().as_u64()) }
}

/// Sets the physical address of the root page table.
///
/// Implemented-By: PI
#[export_name = "__internal_set_root_page_table"]
fn set_root_page_table(address: PhysAddr) {
    // Safety: A valid PhysAddr from the common crate is also valid for the x86_64 crate.
    let address = address.as_u64();
    assert_eq!(address & 4095, 0, "New Root Page table is not aligned");

    let (_, flags) = Cr3::read_raw();
    let value = address | flags as u64;

    unsafe { asm!("mov cr3, {}", in(reg) value, options(nostack, preserves_flags)) };
}
