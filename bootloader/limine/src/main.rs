#![no_std]
#![no_main]

mod acpi;
mod framebuffer;
mod memory_map;
mod stack;

use runner::interface::ModuleInterface;

/// Entrypoint for the kernel.
/// - Creates the module interface.
/// - Runs the module runner.
/// - Starts the service stack.
fn kernel_main() -> ! {
    let iface = ModuleInterface {
        stack_info: stack::get_stack_info(),
        rsdp_address: acpi::get_rsdp_address(),
        framebuffer_info: framebuffer::get_framebuffer_info(),
        memory_map_info: memory_map::get_memory_map_info(),
        memory_info: memory_map::get_memory_info(),
    };

    runner::run_modules(&iface);

    loop {
        core::hint::spin_loop();
    }
}
