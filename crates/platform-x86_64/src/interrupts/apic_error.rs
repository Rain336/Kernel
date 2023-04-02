use log::warn;
use x86_64::structures::idt::InterruptStackFrame;

use crate::devices::{ErrorStatus, LOCAL_APIC};

pub const APIC_ERROR_INTERRUPT_INDEX: u8 = 38;

pub extern "x86-interrupt" fn apic_error(_: InterruptStackFrame) {
    let error = LOCAL_APIC.read_error_status();
    if error.contains(ErrorStatus::REDIRECTABLE_IPI) {
        warn!("APIC Error: Could not send IPI: lowest-priority not supported")
    }
    if error.contains(ErrorStatus::SEND_ILLEGAL_VECTOR) {
        warn!("APIC Error: Could not send IPI: Trying to send to illegeal vector (0-15)")
    }
    if error.contains(ErrorStatus::RECEIVE_ILLEGAL_VECTOR) {
        warn!("APIC Error: Could not receive IPI: Trying to receive into illegeal vector (0-15)")
    }
    if error.contains(ErrorStatus::ILLEGAL_REGISTER_ADDRESS) {
        warn!("APIC Error: Atempt to access reserved register")
    }

    LOCAL_APIC.end_of_interrupt()
}
