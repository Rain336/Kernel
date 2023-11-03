use x86_64::registers::model_specific::Msr;

const IA32_APIC_BASE: Msr = Msr::new(0x1B);

/// Reads the APIC MMIO Address
pub fn read_apic_base() -> u64 {
    let value = unsafe { IA32_APIC_BASE.read() };
    value & 0xFFFFFF000
}

/// Enables the APIC in the IA32_APIC_BASE MSR and sets it into x2APIC mode, if avilable
pub fn enable_apic(x2apic: bool) {
    let mut value = unsafe { IA32_APIC_BASE.read() };
    value |= 0x800;
    if x2apic {
        value |= 0x400;
    }
    #[allow(const_item_mutation)]
    unsafe {
        IA32_APIC_BASE.write(value)
    }
}
