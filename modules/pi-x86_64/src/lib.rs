#![no_std]
#![feature(abi_x86_interrupt)]

//mod counter;
mod devices;
mod gdt;
mod idt;
mod interrupts;
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

    x86_64::instructions::interrupts::enable();
}
