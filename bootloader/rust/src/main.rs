// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![no_std]
#![no_main]

mod acpi;
mod framebuffer;
mod memory;
mod stack;

use bootloader_api::config::Mapping;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::arch::asm;
use runner::interface::{ModuleInterface, PRIMARY_STACK_SIZE};

static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();

    config.kernel_stack_size = PRIMARY_STACK_SIZE as u64;
    //config.mappings.physical_memory = Some(Mapping::FixedAddress(0));
    config.mappings.aslr = cfg!(not(debug_assertions));

    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

pub fn kernel_main(info: &'static mut BootInfo) -> ! {
    let stack_top: u64;
    unsafe {
        asm!(
            "MOV {}, RSP",
            out(reg) stack_top,
            options(nomem, nostack)
        )
    };

    let iface = ModuleInterface {
        stack_info: stack::get_stack_info(stack_top),
        rsdp_address: acpi::get_rsdp_address(info),
        framebuffer_info: framebuffer::get_framebuffer_info(info),
        memory_map_info: memory::get_memory_map_info(info),
        memory_info: memory::get_memory_info(),
    };

    runner::run_modules(&iface);

    loop {
        core::hint::spin_loop();
    }
}
