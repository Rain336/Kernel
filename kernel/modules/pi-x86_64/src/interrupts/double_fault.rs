// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, error: u64) -> ! {
    panic!(
        "Double Fault - {:#x}
RIP = {:#x} CS = {:#x}
RSP = {:#x} SS = {:#x}
RFLAGS = {:#b}",
        error,
        frame.instruction_pointer,
        frame.code_segment,
        frame.stack_pointer,
        frame.stack_segment,
        frame.cpu_flags
    )
}
