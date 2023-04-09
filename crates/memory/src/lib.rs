//! # The Memory Subsystem
//!
//! - [`addr`] Contains exntesions to the address types from [`common::addr`].
//! - [`frame`] Contains the [`frame::PhysFrame`] struct to denote a mappable region of physical memory.
//! - [`kernel`] Contains memory management constructs used by the kernel.
//! - [`page`] Contains the [`page::Page`] struct to denote a mappable region of virtual memory.
//! - [`page_table`] Contains the [`page_table::PageTable`] struct, wich represents a archetecture specific page table.
//! - [`size`] Contains types for the diffrent sizes that are vaild to be mapped to physical memory.
//! - [`translation`] Translates a physical to a virtual address or a virtual to a physical address.
#![no_std]
#![feature(once_cell)]

extern crate alloc;

pub mod addr;
pub mod frame;
pub mod kernel;
pub mod page;
pub mod page_table;
pub mod size;
pub mod translation;

pub struct AddressNotAligned;
