// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use log::warn;
use x86_64::structures::idt::InterruptStackFrame;

use crate::devices::apic::{ErrorStatus, LOCAL_APIC};

pub const APIC_ERROR_INTERRUPT_INDEX: u8 = 38;

pub extern "x86-interrupt" fn apic_error(_: InterruptStackFrame) {
    let error = LOCAL_APIC.read_error_status();
    if error.contains(ErrorStatus::REDIRECTABLE_IPI) {
        warn!("APIC Error: Could not send IPI: lowest-priority not supported")
    }
    if error.contains(ErrorStatus::SEND_ILLEGAL_VECTOR) {
        warn!("APIC Error: Could not send IPI: Trying to send to illegal vector (0-15)")
    }
    if error.contains(ErrorStatus::RECEIVE_ILLEGAL_VECTOR) {
        warn!("APIC Error: Could not receive IPI: Trying to receive into illegal vector (0-15)")
    }
    if error.contains(ErrorStatus::ILLEGAL_REGISTER_ADDRESS) {
        warn!("APIC Error: Attempt to access reserved register")
    }

    LOCAL_APIC.end_of_interrupt()
}
