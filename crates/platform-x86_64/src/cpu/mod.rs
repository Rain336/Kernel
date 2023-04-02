mod gdt;
mod idt;
mod registers;

use crate::devices::LOCAL_APIC;
use x86_64::instructions::interrupts;

pub fn cpu_init(stack_end: u64) {
    gdt::load(stack_end);
    idt::load();
    registers::init();
    LOCAL_APIC.init();

    interrupts::enable();
}
