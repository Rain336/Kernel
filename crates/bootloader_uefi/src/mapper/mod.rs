#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

pub const KERNEL_CODE_START: u64 = 0xFFFFFFFFC0000000;
pub const KERNEL_DATA_START: u64 = 0xFFFFFFFF80000000;

