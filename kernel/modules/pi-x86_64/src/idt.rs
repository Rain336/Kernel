// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::interrupts::double_fault;
use common::sync::SyncLazy;
use x86_64::structures::idt::InterruptDescriptorTable;

static INTERRUPT_DISCRIPTOR_TABLE: SyncLazy<InterruptDescriptorTable> = SyncLazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(1)
    };

    idt
});

pub fn load() {
    log::info!("Loading Interrupt Descriptor Table...");
    INTERRUPT_DISCRIPTOR_TABLE.load();
}
