// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Microdragon Logging Module
//!
//! The logging system provides an implementation for the `log` crate for the rest of the kernel to use.
//! It logs to two different outputs, if available:
//!
//! `Serial Port`
//! By default, microdragon will log to serial port 1 with colored output using ANSI escape sequences.
//! (TODO: Make port and logging configurable)
//!
//! `Framebuffer Terminal`
//! By default, microdragon will request a frame buffer from the bootloader that, if available, will be used for logging.
//! (TODO: Make logging configurable)
#![no_std]

mod escape;
mod framebuffer;
mod position;
mod serial;
mod terminal;
mod theme;

use crate::terminal::TERMINAL_OUTPUT;
use common::sync::{CriticalSection, Spinlock};
use core::fmt::Write;
use interface::ModuleInterface;
use log::{info, Level, LevelFilter, Log, Metadata, Record};
use serial::SERIAL_PORT_OUTPUT;

/// The central [`log::Log`] implementation.
/// There can only be one active Log implementation,
/// so this struct formats the messages and relays them to the outputs.
struct LoggingSubsystem;

impl Log for LoggingSubsystem {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO: For now we always accept logging, but if serial logging is disable,
        // we only want info or higher to display to the framebuffer terminal.
        true
    }

    fn log(&self, record: &Record) {
        // Pre-format the level text.
        let level = match record.level() {
            Level::Error => "[91mERROR[39m",
            Level::Warn => "[93m WARN[39m",
            Level::Info => "[92m INFO[39m",
            Level::Debug => "[94mDEBUG[39m",
            Level::Trace => "[95mTRACE[39m",
        };

        // Start a critical section, since interrupts might log too.
        let _section = CriticalSection::new();

        // Write to logger outputs.
        write_to_output(&SERIAL_PORT_OUTPUT, level, record);
        write_to_output(&TERMINAL_OUTPUT, level, record);
    }

    fn flush(&self) {}
}

static INSTANCE: LoggingSubsystem = LoggingSubsystem;

/// Initializes the logging module.
/// Interrupts should still be disables while this is run.
pub fn init(iface: &ModuleInterface) {
    // Run the initialization sequence for the logging outputs.
    SERIAL_PORT_OUTPUT.lock().init();

    if let Some(fb) = &iface.framebuffer_info {
        TERMINAL_OUTPUT.lock().init(fb);
    }

    // Set global Log implementation.
    let _ = log::set_logger(&INSTANCE);

    // Set global max log level.
    #[cfg(debug_assertions)]
    log::set_max_level(LevelFilter::Trace);
    #[cfg(not(debug_assertions))]
    log::set_max_level(LevelFilter::Info);

    info!("Logging start");
}

/// Called after the kernel memory manager (KMM) has been initialized to correct the physical to virtual address mapping.
pub fn rewire() {
    TERMINAL_OUTPUT.lock().rewire();

    info!("Logging rewired");
}

/// Writes the given record to `output` using pre-formatted `level`.
fn write_to_output<T: Write>(output: &Spinlock<T>, level: &str, record: &Record) {
    // Lock the output.
    let mut guard = output.lock();

    // write using `writeln` macro.
    let _ = writeln!(
        guard,
        "{} {}@{} {}",
        level,
        record
            .file()
            .or_else(|| record.module_path())
            .unwrap_or_default(),
        record.line().unwrap_or_default(),
        record.args()
    );
}
