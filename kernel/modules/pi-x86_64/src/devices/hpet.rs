// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::counter::Counter;
use acpi::HpetTable;
use core::ptr;
use log::{info, warn};
use common::sync::SyncLazy;

pub static HIGH_PRECISION_EVENT_TIMER: SyncLazy<Option<HighPrecisionEventTimer>> =
    SyncLazy::new(|| {
        let table = acpi::find_table::<HpetTable>()?;
        if table.address.address_space_id != 0 {
            warn!("HPET not in System Memory. Cannot use it");
            return None;
        }

        let hpet = HighPrecisionEventTimer::new(table.address.address);

        info!(
            "HPET Clock Speed: {}Hz 64-bit Counter: {} Timer Count: {}",
            hpet.frequency(),
            hpet.is_64bit(),
            hpet.timer_count()
        );

        Some(hpet)
    });

pub struct HighPrecisionEventTimer {
    address: u64,
    frequency: u64,
    is_64bit: bool,
    timers: u8,
}

impl HighPrecisionEventTimer {
    const PHZ_TO_HZ: u64 = 1000000000000000;
    const GENERAL_CONFIGURATION_REGISTER_OFFSET: u64 = 0x010;
    const MAIN_COUNTER_VALUE_REGISTER_OFFSET: u64 = 0x0F0;

    fn new(address: u64) -> Self {
        let capabilites = unsafe { ptr::read_volatile(address as *const u64) };
        let frequency = Self::PHZ_TO_HZ / (capabilites >> 32);
        let is_64bit = (capabilites & 0x2000) == 0x2000;
        let timers = ((capabilites & 0x1F00) >> 8) as u8;

        unsafe {
            ptr::write_volatile(
                (address + Self::MAIN_COUNTER_VALUE_REGISTER_OFFSET) as *mut u64,
                0,
            );

            let mut configuration = ptr::read_volatile(
                (address + Self::GENERAL_CONFIGURATION_REGISTER_OFFSET) as *const u64,
            );
            configuration = (configuration & !0b11) | 1;
            ptr::write_volatile(
                (address + Self::GENERAL_CONFIGURATION_REGISTER_OFFSET) as *mut u64,
                configuration,
            );
        }

        HighPrecisionEventTimer {
            address,
            frequency,
            is_64bit,
            timers,
        }
    }

    pub fn is_64bit(&self) -> bool {
        self.is_64bit
    }

    pub fn timer_count(&self) -> u8 {
        self.timers
    }

    pub fn value(&self) -> u64 {
        unsafe {
            ptr::read_volatile(
                (self.address + Self::MAIN_COUNTER_VALUE_REGISTER_OFFSET) as *const u64,
            )
        }
    }
}

impl Counter for HighPrecisionEventTimer {
    fn frequency(&self) -> u64 {
        self.frequency
    }

    fn calibrate(&self) {}
}
