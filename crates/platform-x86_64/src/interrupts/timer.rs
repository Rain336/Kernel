use crate::devices::LOCAL_APIC;
use x86_64::structures::idt::InterruptStackFrame;

pub const TIMER_INTERRUPT_INDEX: u8 = 33;

pub extern "x86-interrupt" fn timer(frame: InterruptStackFrame) {
    LOCAL_APIC.end_of_interrupt()
}
