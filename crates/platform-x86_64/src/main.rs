#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod counter;
mod cpu;
mod devices;
mod interrupts;

use common::sync::SyncLazy;
use core::panic::PanicInfo;
use log::{error, info};
use raw_cpuid::CpuId;

pub static CPUID: SyncLazy<CpuId> = SyncLazy::new(CpuId::new);

#[no_mangle]
extern "C" fn _start() -> ! {
    logging::init();
    //cpu::cpu_init(info.kernel_position.stack_end);
    //acpi::init(info.sdt_address);
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
