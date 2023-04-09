/// The size of the kernel's primary stacks in bytes.
pub const PRIMARY_STACK_SIZE: usize = 64 * 1024;

/// The size of the kernel's secondary stacks in bytes.
pub const SECONDARY_STACK_SIZE: usize = 16 * 1024;

/// The kernel's primary stack for the bootstrap processor.
static mut BOOTSTRAP_PRIMARY_STACK: &mut [u8] = &mut [0; PRIMARY_STACK_SIZE];

/// The kernel's secondary stack for the bootstrap processor.
static mut BOOTSTRAP_SECONDARY_STACK: &mut [u8] = &mut [0; SECONDARY_STACK_SIZE];

pub fn get_bootstrap_primary_stack() -> u64 {
    unsafe { BOOTSTRAP_PRIMARY_STACK.as_ptr_range().end as u64 }
}

pub fn get_bootstrap_secondary_stack() -> u64 {
    unsafe { BOOTSTRAP_SECONDARY_STACK.as_ptr_range().end as u64 }
}

#[no_mangle]
extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "MOV RSP, {}",
            "MOV RBP, RSP",
            in(reg) get_bootstrap_primary_stack()
        );
    }

    crate::kernel_main()
}
