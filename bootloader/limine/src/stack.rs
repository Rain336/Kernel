// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use runner::interface::{StackInfo, PRIMARY_STACK_SIZE, SECONDARY_STACK_SIZE};

/// The kernel's primary stack for the bootstrap processor.
static mut BOOTSTRAP_PRIMARY_STACK: &mut [u8] = &mut [0; PRIMARY_STACK_SIZE];

/// The kernel's secondary stack for the bootstrap processor.
static mut BOOTSTRAP_SECONDARY_STACK: &mut [u8] = &mut [0; SECONDARY_STACK_SIZE];

unsafe fn get_bootstrap_primary_stack() -> *const u8 {
    BOOTSTRAP_PRIMARY_STACK.as_ptr().add(PRIMARY_STACK_SIZE)
}

/// Creates the [`StackInfo`] struct for the module interface.
pub fn get_stack_info() -> StackInfo {
    unsafe {
        StackInfo {
            primary_stack: BOOTSTRAP_PRIMARY_STACK.as_ptr() as u64,
            secondary_stack: BOOTSTRAP_SECONDARY_STACK.as_ptr() as u64,
        }
    }
}

/// Hidden kernel entrypoint which just switches to our own stack and calls [`crate::kernel_main`].
#[no_mangle]
extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "MOV RSP, {}",
            "MOV RBP, RSP",
            in(reg) get_bootstrap_primary_stack()
        );
    }

    crate::kernel_main()
}
