// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::devices::apic::LOCAL_APIC;
use x86_64::structures::idt::InterruptStackFrame;

pub const TIMER_INTERRUPT_INDEX: u8 = 33;

pub extern "x86-interrupt" fn timer(frame: InterruptStackFrame) {
    LOCAL_APIC.end_of_interrupt()
}
