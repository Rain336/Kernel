#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod counter;
mod cpu;
mod devices;
mod interrupts;
mod stack;

use common::sync::SyncLazy;
use core::panic::PanicInfo;
use log::{error, info};
use raw_cpuid::CpuId;

pub static CPUID: SyncLazy<CpuId> = SyncLazy::new(CpuId::new);

fn kernel_main() -> ! {
    logging::init();
    cpu::cpu_init(
        stack::get_bootstrap_primary_stack(),
        stack::get_bootstrap_secondary_stack(),
    );
    //acpi::init();
    //counter::init();
    //info!("Counter Frequency: {}Hz", counter::frequency());

    info!("That's it for now...");
    loop {
        x86_64::instructions::hlt()
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);

    loop {
        x86_64::instructions::hlt()
    }
}
