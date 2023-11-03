/// A critical section that disables external interrupts until it's dropped.
/// Critical sections can be nested without interfering with each other.
pub struct CriticalSection(bool);

impl CriticalSection {
    /// Starts a new critical section.
    pub fn new() -> Self {
        #[cfg(target_arch = "x86_64")]
        if x86_64::instructions::interrupts::are_enabled() {
            x86_64::instructions::interrupts::disable();
            CriticalSection(true)
        } else {
            CriticalSection(false)
        }

        #[cfg(target_arch = "aarch64")]
        todo!();

        #[cfg(riscv)]
        if riscv::register::mstatus::read().mie() {
            riscv::register::mstatus::clear_mie();
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
            #[cfg(target_arch = "x86_64")]
            x86_64::instructions::interrupts::enable();

            #[cfg(target_arch = "aarch64")]
            todo!();

            #[cfg(riscv)]
            riscv::register::mstatus::set_mie();
        }
    }
}
