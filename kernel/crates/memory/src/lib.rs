// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # The Memory Subsystem
//!
//! - [`addr`] Contains extensions to the address types from [`common::addr`].
//! - [`frame`] Contains the [`frame::PhysFrame`] struct to denote a mappable region of physical memory.
//! - [`page`] Contains the [`page::Page`] struct to denote a mappable region of virtual memory.
//! - [`page_table`] Contains the [`page_table::PageTable`] struct, which represents a architecture specific page table.
//! - [`size`] Contains types for the different sizes that are valid to be mapped to physical memory.
//! - [`translation`] Translates a physical to a virtual address or a virtual to a physical address.
//!
#![no_std]

pub mod addr;
pub mod frame;
pub mod page;
pub mod page_table;
pub mod size;

pub struct AddressNotAligned;
