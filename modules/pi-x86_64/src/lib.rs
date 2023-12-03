// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![no_std]
#![feature(abi_x86_interrupt)]

//mod counter;
mod devices;
mod gdt;
mod idt;
mod interrupts;
mod magic;
mod registers;

use common::sync::SyncLazy;
use interface::ModuleInterface;
use raw_cpuid::{CpuId, CpuIdReaderNative};

static CPUID: SyncLazy<CpuId<CpuIdReaderNative>> = SyncLazy::new(CpuId::new);

pub fn init(iface: &ModuleInterface) {
    gdt::load(
        iface.stack_info.primary_stack,
        iface.stack_info.secondary_stack,
    );
    idt::load();
    registers::init();

    common::interrupts::enable();
}
