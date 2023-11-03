// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use super::Interrupts;
use core::arch::asm;
use core::sync::atomic::{compiler_fence, Ordering};

pub enum PlatformInterrupts {}

impl Interrupts for PlatformInterrupts {
    fn are_enabled() -> bool {
        let flags: u64;
        unsafe { asm!("pushfq; pop {}", out(reg) flags, options(nomem, preserves_flags)) };
        (flags & (1 << 9)) != 0
    }

    fn enable() {
        compiler_fence(Ordering::Release);
        unsafe { asm!("sti", options(nomem, nostack)) }
    }

    fn disable() {
        compiler_fence(Ordering::Acquire);
        unsafe { asm!("cli", options(nomem, nostack)) }
    }
}
