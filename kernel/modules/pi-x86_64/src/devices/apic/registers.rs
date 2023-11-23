// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#[repr(u64)]
pub enum LocalApicRegister {
    Id = 0x20,
    Version = 0x30,
    TaskPriority = 0x80,
    ArbitrationPriority = 0x90,
    ProcessorPriority = 0xA0,
    EndOfInterrupt = 0xB0,
    RemoteRead = 0xC0,
    LocalDestination = 0xD0,
    DestinationFormat = 0xE0,
    SpuriousInterruptVector = 0xF0,
    ErrorStatus = 0x280,
    LvtCmic = 0x2F0,
    InterruptCommandLow = 0x300,
    InterruptCommandHigh = 0x310,
    LvtTimer = 0x320,
    LvtThermalSensor = 0x330,
    LvtPerformanceMonitoringCounters = 0x340,
    LvtLint0 = 0x350,
    LvtLint1 = 0x360,
    LvtError = 0x370,
    TimerInitialCount = 0x380,
    TimerCurrentCount = 0x390,
    TimerDevideConfiguration = 0x3E0,
}
