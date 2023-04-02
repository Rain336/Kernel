use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, error: u64) -> ! {
    panic!(
        "Double Fault - {:#x}\nRIP = {:#x} CS = {:#x}\nRSP = {:#x} SS = {:#x}\nRFLAGS = {:#b}",
        error,
        frame.instruction_pointer,
        frame.code_segment,
        frame.stack_pointer,
        frame.stack_segment,
        frame.cpu_flags
    )
}
