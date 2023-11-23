// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::sync::Spinlock;

/// For serial logging we can just use the [`SerialPort`] type from the [`uart_16550`] crate.
/// It needs to wrapped into a spinlock.
#[cfg(target_arch = "x86_64")]
pub static SERIAL_PORT_OUTPUT: Spinlock<uart_16550::SerialPort> =
    Spinlock::new(unsafe { uart_16550::SerialPort::new(config::value!("serial.port")) });
