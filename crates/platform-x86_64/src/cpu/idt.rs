use crate::interrupts::{
    apic_error, double_fault, timer, APIC_ERROR_INTERRUPT_INDEX, TIMER_INTERRUPT_INDEX,
};
use log::debug;
use common::sync::SyncLazy;
use x86_64::structures::idt::InterruptDescriptorTable;

static INTERRUPT_DISCRIPTOR_TABLE: SyncLazy<InterruptDescriptorTable> = SyncLazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(1)
    };
    idt[TIMER_INTERRUPT_INDEX as usize].set_handler_fn(timer);
    idt[APIC_ERROR_INTERRUPT_INDEX as usize].set_handler_fn(apic_error);
    idt
});

pub fn load() {
    debug!("Loading Interrupt Descriptor Table...");
    INTERRUPT_DISCRIPTOR_TABLE.load();
}
