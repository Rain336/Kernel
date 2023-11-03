mod base;
mod error;
mod ipi;
mod registers;

use crate::interrupts::{APIC_ERROR_INTERRUPT_INDEX, TIMER_INTERRUPT_INDEX};
use crate::CPUID;
use common::sync::SyncLazy;
use core::arch::x86_64::_rdtsc;
use registers::LocalApicRegister;
use x86_64::registers::model_specific::Msr;

pub use self::error::ErrorStatus;
pub use self::ipi::*;

pub static LOCAL_APIC: SyncLazy<LocalApic> = SyncLazy::new(LocalApic::new);

fn has_features() -> (bool, bool) {
    CPUID
        .get_feature_info()
        .map(|x| (x.has_x2apic(), x.has_tsc_deadline()))
        .unwrap_or_default()
}

const fn timer_lvt(vector: u8, tsc_deadline: bool) -> u32 {
    if tsc_deadline {
        vector as u32 | 0x40000
    } else {
        vector as u32
    }
}

pub struct LocalApic {
    address: u64,
    x2apic: bool,
    tsc_deadline: bool,
    cmci: bool,
    eoi_boradcast: bool,
}

impl LocalApic {
    pub fn new() -> Self {
        let (x2apic, tsc_deadline) = has_features();

        LocalApic {
            address: base::read_apic_base(),
            x2apic,
            tsc_deadline,
            cmci: false,
            eoi_boradcast: false,
        }
        .collect_info()
    }

    pub fn init(&self) {
        base::enable_apic(self.x2apic);

        self.write(LocalApicRegister::TaskPriority, 0);
        self.write(LocalApicRegister::LocalDestination, 0);
        if !self.x2apic {
            self.write(LocalApicRegister::DestinationFormat, 0);
        }

        // CMCI was introduced with Intel Xeon 5500, so we have check that before writing.
        // All others are either Pentium 4 or older
        // TODO: self.write(LocalApicRegister::LvtCmic, todo!());
        self.write(
            LocalApicRegister::LvtTimer,
            timer_lvt(TIMER_INTERRUPT_INDEX, self.tsc_deadline),
        );
        // TODO: self.write(LocalApicRegister::LvtThermalSensor, todo!());
        // TODO: self.write(LocalApicRegister::LvtPerformanceMonitoringCounters, todo!());
        // TODO: self.write(LocalApicRegister::LvtLint0, todo!());
        // TODO: self.write(LocalApicRegister::LvtLint1, todo!());
        self.write(
            LocalApicRegister::LvtError,
            APIC_ERROR_INTERRUPT_INDEX as u32,
        );

        //let mut svr = todo!() | 0x100;

        //if self.eoi_boradcast {
        //    svr |= 0x1000;
        //}

        //self.write(LocalApicRegister::SpuriousInterruptVector, svr);
    }

    pub fn end_of_interrupt(&self) {
        self.write(LocalApicRegister::EndOfInterrupt, 0);
    }

    pub fn send(&self, ipi: &InterProcessorInterrupt) {
        if self.x2apic {
            if let InterProcessorInterrupt::OnlySelf { vector } = ipi {
                let mut msr = Msr::new(0x83F);
                unsafe { msr.write(*vector as u64) };
                return;
            }

            let mut msr = Msr::new(0x830);
            unsafe { msr.write(ipi.encode()) };
        } else {
            self.write(LocalApicRegister::InterruptCommandHigh, ipi.high());
            self.write(LocalApicRegister::InterruptCommandLow, ipi.low());
        }
    }

    pub fn arm_timer(&self, ticks: u32) {
        if self.tsc_deadline {
            unsafe { Msr::new(0x6E0).write(_rdtsc() + ticks as u64) }
        } else {
            self.write(LocalApicRegister::TimerInitialCount, ticks);
        }
    }

    pub fn disarm_timer(&self) {
        if self.tsc_deadline {
            unsafe { Msr::new(0x6E0).write(0) }
        } else {
            self.write(LocalApicRegister::TimerInitialCount, 0);
        }
    }

    pub fn read_error_status(&self) -> ErrorStatus {
        self.write(LocalApicRegister::ErrorStatus, 0);
        ErrorStatus::from_bits_truncate(self.read(LocalApicRegister::ErrorStatus))
    }

    pub fn id(&self) -> u32 {
        if self.x2apic {
            self.read(LocalApicRegister::Id)
        } else {
            self.read(LocalApicRegister::Id) >> 24
        }
    }

    fn collect_info(mut self) -> Self {
        let version = self.read(LocalApicRegister::Version);
        self.cmci = ((version & 0xFF0000) >> 16) == 6;
        self.eoi_boradcast = (version & 0x1000000) == 0x1000000;
        self
    }

    fn read(&self, offset: LocalApicRegister) -> u32 {
        if self.x2apic {
            let msr = ((offset as u32) >> 4) + 0x800;
            unsafe { Msr::new(msr).read() as u32 }
        } else {
            unsafe { ((self.address + offset as u64) as *const u32).read_volatile() }
        }
    }

    fn write(&self, offset: LocalApicRegister, value: u32) {
        if self.x2apic {
            let msr = ((offset as u32) >> 4) + 0x800;
            unsafe { Msr::new(msr).write(value as u64) }
        } else {
            unsafe { ((self.address + offset as u64) as *mut u32).write_volatile(value) };
        }
    }
}
