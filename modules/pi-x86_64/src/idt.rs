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
