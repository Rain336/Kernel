//! # Module Runner
//!
//! This modules singe purpose is to tie all other modules together and run them in order.
//! It supplies a singe function [`run_modules`] to run all modules with the supplied [`ModuleInterface`].
//! It also exports the module interface it uses, so runner and interface are always synced up on their interface.
//!
#![no_std]

pub extern crate interface;

use core::panic::PanicInfo;
use interface::ModuleInterface;
use log::{error, info};

/// Runs all microdragon modules with the given [`ModuleInterface`].
pub fn run_modules(iface: &ModuleInterface) {
    logging::init(iface);
    pi::init(iface);
    kmm::init(iface);
    logging::rewire();
    acpi::init(iface);

    info!("Modules loaded!");
}

/// Dummy Panic Handler, to be moved.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info);

    loop {
        core::hint::spin_loop();
    }
}
