//! # High Precision Event Timer (HPET)
//! The HPET is a pre-calibrated timer found in most x86 PCs.
//! The HPET has either a 32- or 64-Bit Counter Register, that is incremented at a fixed rate.
//!
//! ## Legacy Replacement Mode
//! The HPET can be used to emulate the PIT and RTC Timer using legacy replacement mode, which is done by a lot of firmwares.
//! This mode uses up the first two comparators of the HPET for PIT and RTC Timer respectively.
//! Even though these comparators can be adjusted, this library restrains from doing so.
//! Legacy replacement mode can be disabled though.
#![no_std]
#![warn(missing_docs)]

mod comparator;
mod registers;
mod timer;

use core::mem;
use core::slice;

pub use comparator::HpetComparator;

/// This is the main data type of this library,
/// keeping a reference to the registers of the HPET and it's comparators.
/// As well as having a mechanism to lend and return comparators.
pub struct HighPrecisionEventTimer {
    registers: &'static mut registers::HpetRegisters,
    timers: &'static mut [timer::HpetTimer],
    tracking: u8,
}

impl HighPrecisionEventTimer {
    /// Creates a new HPET management struct with the registers at the given address.
    /// You can chose whenever to enable Legacy Replacement Mode, if available.
    ///
    /// ## Safety
    /// The data at the given address must be an I/O mapped HPET.
    pub unsafe fn new(address: u64, legacy_replacement: bool) -> HighPrecisionEventTimer {
        let registers = unsafe { &mut *(address as *mut registers::HpetRegisters) };
        let timers =
            (address + mem::size_of::<registers::HpetRegisters>() as u64) as *mut timer::HpetTimer;
        let gci = registers.general_capabilitis_and_id_register();
        let timers = unsafe { slice::from_raw_parts_mut(timers, ((gci >> 8) & 0b11111) as usize) };

        let tracking = if (gci & 0x8000) == 0x8000 {
            if legacy_replacement {
                registers.set_general_configuration_register(
                    registers.general_configuration_register() & 2,
                );
                0b11
            } else {
                registers.set_general_configuration_register(
                    registers.general_configuration_register() & !2,
                );
                0
            }
        } else {
            0
        };

        HighPrecisionEventTimer {
            registers,
            timers,
            tracking,
        }
    }

    /// Gets the revision
    pub fn rev_id(&self) -> u8 {
        self.registers.general_capabilitis_and_id_register() as u8
    }

    /// Get the number of comparators offered by the HPET
    /// Two comparators cannot be used, if legacy replacement mode is used.
    pub fn comparator_count(&self) -> u8 {
        ((self.registers.general_capabilitis_and_id_register() >> 8) & 0b11111) as u8
    }

    /// Returns whenever the counter of the HPET is 64 bits wide.
    pub fn counter_is_64_bit(&self) -> bool {
        (self.registers.general_capabilitis_and_id_register() & 0x2000) == 0x2000
    }

    /// Returns whenever legacy replacement mode is supported by this HPET.
    pub fn supports_legacy_replacement_mode(&self) -> bool {
        (self.registers.general_capabilitis_and_id_register() & 0x8000) == 0x8000
    }

    /// The PCI Vendor Id of the HPET.
    pub fn vendor_id(&self) -> u16 {
        (self.registers.general_capabilitis_and_id_register() >> 16) as u16
    }

    /// This gives the freqency of the HPET in femptoseconds (10^-15).
    /// The value must be less than or equal to `0x05F5E100` (10^8 femptoseconds = 100 nanoseconds).
    pub fn main_counter_tick_period(&self) -> u32 {
        (self.registers.general_capabilitis_and_id_register() >> 32) as u32
    }

    /// Returns whenever the HPET is enabled.
    /// When the HPET is disabled, the counter won't increment and no interrupts will be generated.
    pub fn is_enabled(&self) -> bool {
        (self.registers.general_configuration_register() & 1) == 1
    }

    /// Enables the HPET, allowing the counter to increment and interrupts to generate.
    pub fn enable(&mut self) {
        let value = self.registers.general_configuration_register() | 1;
        self.registers.set_general_configuration_register(value);
    }

    /// Disables the HPET, preventing the counter from incrementing and interrupts won't be generated.
    pub fn disable(&mut self) {
        let value = self.registers.general_configuration_register() & (!1);
        self.registers.set_general_configuration_register(value);
    }

    /// Get the current value of the counter.
    pub fn counter(&self) -> u64 {
        self.registers.main_counter_value_register()
    }

    /// Sets the counter to a specific value.
    /// The counter can only be set, while the HPET is disabled.
    pub fn set_counter(&mut self, value: u64) {
        if !self.is_enabled() {
            self.registers.set_main_counter_value_register(value)
        }
    }

    /// Searches for an available comparator, marks it as in use and returns it.
    /// The comparator won't be marked as available again on drop,
    /// so be sure to call [`return_comparator`] with it when you are done.
    /// If no comparator is available, `None` will be returned.
    pub fn lend_comparator(&mut self) -> Option<HpetComparator> {
        let idx = self.tracking.trailing_ones() as usize;
        let timer = self.timers.get_mut(idx)?;

        self.tracking |= 1 << idx;
        Some(HpetComparator::new(timer, idx as u8))
    }

    /// Returns a lended comparator, marking it as available again.
    pub fn return_comparator(&mut self, comparator: HpetComparator) {
        unsafe { self.return_comparator_unchecked(comparator.index()) }
    }

    /// Returns a lended comparator by index, marking it as available again, without checking if the comparator has been lended before.
    ///
    /// ## Safety
    /// Since it cannot be checked if the comparator might still be in use,
    /// a comaprator could be lended out twice, causing a race condition.
    pub unsafe fn return_comparator_unchecked(&mut self, index: u8) {
        let mask = !(1u8 << index);
        self.tracking &= mask;
    }
}
