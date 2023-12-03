// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use runner::interface::{StackInfo, PRIMARY_STACK_SIZE, SECONDARY_STACK_SIZE};

/// The kernel's secondary stack for the bootstrap processor.
static mut BOOTSTRAP_SECONDARY_STACK: &mut [u8] = &mut [0; SECONDARY_STACK_SIZE];

pub fn get_stack_info(stack: u64) -> StackInfo {
    StackInfo {
        primary_stack: stack - PRIMARY_STACK_SIZE as u64,
        secondary_stack: unsafe { BOOTSTRAP_SECONDARY_STACK.as_ptr() as u64 },
    }
}
