//! ## Virtual memory Management (VMM)
//!
//! Virtual memory management for the kernel provides the following:
//! - The mapping of physical frames into virtual pages in the kernel dynamic heap area.
//! - Translation of kernel virtual addresses to physical addresses.
//!
mod mapping;
mod translate;

pub use mapping::*;
pub use translate::*;
