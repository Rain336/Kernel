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
#![feature(proc_macro_hygiene)]

#[config::config(serial.enabled)]
mod serial;
#[config::config(terminal.enabled)]
mod terminal;

use common::sync::{CriticalSection, Spinlock};
use core::fmt::Write;
use interface::ModuleInterface;
use log::{info, Level, LevelFilter, Log, Metadata, Record};

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
            Level::Error => "\x1B[91mERROR\x1B[39m",
            Level::Warn => "\x1B[93m WARN\x1B[39m",
            Level::Info => "\x1B[92m INFO\x1B[39m",
            Level::Debug => "\x1B[94mDEBUG\x1B[39m",
            Level::Trace => "\x1B[95mTRACE\x1B[39m",
        };

        // Start a critical section, since interrupts might log too.
        let _section = CriticalSection::new();

        // Write to logger outputs.
        #[config::config(serial.enabled)]
        write_to_output(&serial::SERIAL_PORT_OUTPUT, level, record);
        #[config::config(terminal.enabled)]
        write_to_output(&terminal::TERMINAL_OUTPUT, level, record);
    }

    fn flush(&self) {}
}

static INSTANCE: LoggingSubsystem = LoggingSubsystem;

/// Initializes the logging module.
/// Interrupts should still be disables while this is run.
pub fn init(iface: &ModuleInterface) {
    // Run the initialization sequence for the logging outputs.
    #[config::config(serial.enabled)]
    serial::SERIAL_PORT_OUTPUT.lock().init();

    if let Some(fb) = &iface.framebuffer_info {
        #[config::config(terminal.enabled)]
        terminal::TERMINAL_OUTPUT.lock().init(fb);
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
    #[config::config(terminal.enabled)]
    terminal::TERMINAL_OUTPUT.lock().rewire();

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
