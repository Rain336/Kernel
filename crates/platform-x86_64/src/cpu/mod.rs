mod gdt;
mod idt;
mod registers;

use crate::devices::LOCAL_APIC;
use x86_64::instructions::interrupts;

pub fn cpu_init(primary_stack: u64, secondary_stack: u64) {
    gdt::load(primary_stack, secondary_stack);
    idt::load();
    registers::init();
    LOCAL_APIC.init();

    interrupts::enable();
}
