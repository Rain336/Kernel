// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
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
