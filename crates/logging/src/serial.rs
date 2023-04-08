use common::sync::Spinlock;
use uart_16550::SerialPort;

/// For serial logging we can just use the [`SerialPort`] type from the [`uart_16550`] crate.
/// It needs to wrapped into a spinlock.
#[cfg(target_arch = "x86_64")]
pub static SERIAL_PORT_OUTPUT: Spinlock<SerialPort> =
    Spinlock::new(unsafe { SerialPort::new(0x3F8) });
