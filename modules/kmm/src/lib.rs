// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## The Kernel Memory Management Module (KMM)
//!
//! The goal of KMM is to setup the MMU so that mapping works as described in [`common::memory`].
//! And to provide a global allocator for the kernel.
//!
//! - [`allocation`] contains the global allocator for the kernel.
//! - [`page_flags`] page flags used by [`vmm`] and [`setup`].
//! - [`pmm`] keeps track of free physical frames that can be allocated by the kernel or userspace processes.
//! - [`setup`] runs on kernel startup and sets up virtual memory according to [`common::memory`].
//! - [`vmm`] maps pages into the kernel data area.
//!
#![no_std]

extern crate alloc;

mod allocation;
mod magic;
mod page_flags;
mod pmm;
mod setup;
mod vmm;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: allocation::GlobalAllocator = allocation::GlobalAllocator;

pub use setup::init;
