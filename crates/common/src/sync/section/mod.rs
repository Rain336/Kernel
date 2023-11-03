// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use x86_64::PlatformInterrupts;

/// A trait implemented by each architecture as [`PlatformInterrupts`] to manage interrupts.
pub trait Interrupts {
    /// Returns whenever interrupts are enabled (unmasked).
    fn are_enabled() -> bool;

    /// Enables (masks) all interrupts.
    fn enable();

    /// Disables (unmasks) all interrupts.
    fn disable();
}

/// A critical section that disables external interrupts until it's dropped.
/// Critical sections can be nested without interfering with each other.
pub struct CriticalSection(bool);

impl CriticalSection {
    /// Starts a new critical section.
    pub fn new() -> Self {
        if PlatformInterrupts::are_enabled() {
            PlatformInterrupts::disable();
            CriticalSection(true)
        } else {
            CriticalSection(false)
        }
    }
}

impl Default for CriticalSection {
    fn default() -> Self {
        CriticalSection::new()
    }
}

impl Drop for CriticalSection {
    fn drop(&mut self) {
        if self.0 {
            PlatformInterrupts::enable();
        }
    }
}
