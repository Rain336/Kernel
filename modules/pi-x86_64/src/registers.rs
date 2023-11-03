use crate::CPUID;
use log::debug;
use common::sync::SyncLazy;
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::xcontrol::{XCr0, XCr0Flags};

pub fn init() {
    load_extended_feature_enable_register();
    load_control_registers();
    load_extended_control_registers();
}

static EXTENDED_FEATURE_ENABLE_REGISTER: SyncLazy<EferFlags> = SyncLazy::new(|| {
    if let Some(info) = CPUID.get_extended_processor_and_feature_identifiers() {
        let mut bits = EferFlags::LONG_MODE_ENABLE;
        if info.has_syscall_sysret() {
            bits |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        }
        if info.has_execute_disable() {
            bits |= EferFlags::NO_EXECUTE_ENABLE;
        }
        if info.has_svm() {
            // AMD only
            bits |= EferFlags::SECURE_VIRTUAL_MACHINE_ENABLE;
        }
        if info.has_fast_fxsave_fxstor() {
            // AMD only
            bits |= EferFlags::FAST_FXSAVE_FXRSTOR;
        }
        bits
    } else {
        EferFlags::LONG_MODE_ENABLE
    }
});

fn load_extended_feature_enable_register() {
    debug!("Efer: {:?}", *EXTENDED_FEATURE_ENABLE_REGISTER);
    #[cfg(debug_assertions)]
    unsafe {
        Efer::write(*EXTENDED_FEATURE_ENABLE_REGISTER)
    }
    #[cfg(not(debug_assertions))]
    unsafe {
        Efer::write_raw(EXTENDED_FEATURE_ENABLE_REGISTER.bits())
    }
}

static CONTROL_REGISTER_4: SyncLazy<Cr4Flags> = SyncLazy::new(|| {
    let mut bits = if let Some(info) = CPUID.get_feature_info() {
        let mut bits = Cr4Flags::PHYSICAL_ADDRESS_EXTENSION | Cr4Flags::OSXMMEXCPT_ENABLE;
        if info.has_de() {
            bits |= Cr4Flags::DEBUGGING_EXTENSIONS;
        }
        if info.has_mce() {
            bits |= Cr4Flags::MACHINE_CHECK_EXCEPTION;
        }
        if info.has_pge() {
            bits |= Cr4Flags::PAGE_GLOBAL;
        }
        //if info.???() {
        //    bits |= Cr4Flags::PERFORMANCE_MONITOR_COUNTER;
        //}
        if info.has_vmx() {
            // Intel only
            bits |= Cr4Flags::VIRTUAL_MACHINE_EXTENSIONS;
        }
        if info.has_smx() {
            // Intel only
            bits |= Cr4Flags::SAFER_MODE_EXTENSIONS;
        }
        if info.has_pcid() {
            bits |= Cr4Flags::PCID;
        }
        if info.has_xsave() {
            bits |= Cr4Flags::OSXSAVE;
        }
        if info.has_fxsave_fxstor() {
            bits |= Cr4Flags::OSFXSR;
        }
        //if info.???() {
        //    bits |= Cr4Flags::CONTROL_FLOW_ENFORCEMENT;
        //}
        bits
    } else {
        Cr4Flags::PHYSICAL_ADDRESS_EXTENSION | Cr4Flags::OSXMMEXCPT_ENABLE
    };

    if let Some(info) = CPUID.get_extended_feature_info() {
        // Enabling this will break everything since we don't support L5 paging yet
        //if info.has_la57() { (INTEL SUPPORTED / AMD UNKNOWN)
        //    bits |= Cr4Flags::L5_PAGING;
        //}
        if info.has_fsgsbase() {
            bits |= Cr4Flags::FSGSBASE;
        }
        if info.has_smep() {
            bits |= Cr4Flags::SUPERVISOR_MODE_EXECUTION_PROTECTION;
        }
        // Might break some stuff, but should definitely be considers in the future
        //if info.has_smap() {
        //    bits |= Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION;
        //}
        if info.has_pku() {
            // Intel only
            bits |= Cr4Flags::PROTECTION_KEY_USER;
        }
        //if info.???() { Should be ECX bit 23 but not in raw_cpuid? Only mention on Wikipedia
        //    bits |= Cr4Flags::KEY_LOCKER;
        //}
        //if info.???() { Should be ECX bit 31 but not in raw_cpuid? Only mention on Wikipedia
        //    bits |= Cr4Flags::PROTECTION_KEY_SUPERVISOR;
        //}
        if info.has_umip() {
            // Intel only
            bits |= Cr4Flags::USER_MODE_INSTRUCTION_PREVENTION;
        }
    }

    bits
});

fn load_control_registers() {
    unsafe {
        Cr0::write(
            Cr0Flags::PAGING
                | Cr0Flags::WRITE_PROTECT
                | Cr0Flags::NUMERIC_ERROR
                | Cr0Flags::MONITOR_COPROCESSOR
                | Cr0Flags::PROTECTED_MODE_ENABLE,
        );
    }

    debug!("Cr4: {:?}", *CONTROL_REGISTER_4);
    #[cfg(debug_assertions)]
    unsafe {
        Cr4::write(*CONTROL_REGISTER_4)
    }
    #[cfg(not(debug_assertions))]
    unsafe {
        Cr4::write_raw(CONTROL_REGISTER_4.bits())
    }
}

static EXTENDED_CONTROL_REGISTER: SyncLazy<XCr0Flags> = SyncLazy::new(|| {
    if let Some(info) = CPUID.get_extended_state_info() {
        debug!("{:?}", info);
        let mut bits = XCr0Flags::X87;
        if info.xcr0_supports_sse_128() {
            bits |= XCr0Flags::SSE;
        }
        if info.xcr0_supports_avx_256() {
            bits |= XCr0Flags::AVX;
        }
        if info.xcr0_supports_mpx_bndregs() && info.xcr0_supports_mpx_bndcsr() {
            bits |= XCr0Flags::BNDREG | XCr0Flags::BNDCSR;
        }
        if info.xcr0_supports_avx512_opmask()
            && info.xcr0_supports_avx512_zmm_hi256()
            && info.xcr0_supports_avx512_zmm_hi16()
        {
            bits |= XCr0Flags::OPMASK | XCr0Flags::ZMM_HI256 | XCr0Flags::HI16_ZMM;
        }
        if info.xcr0_supports_pkru() {
            bits |= XCr0Flags::MPK;
        }
        bits
    } else {
        XCr0Flags::X87
    }
});

fn load_extended_control_registers() {
    if !CONTROL_REGISTER_4.contains(Cr4Flags::OSXSAVE) {
        return;
    }

    debug!("XCr0: {:?}", *EXTENDED_CONTROL_REGISTER);
    #[cfg(debug_assertions)]
    unsafe {
        XCr0::write(*EXTENDED_CONTROL_REGISTER)
    }
    #[cfg(not(debug_assertions))]
    unsafe {
        XCr0::write_raw(EXTENDED_CONTROL_REGISTER.bits())
    }
}
