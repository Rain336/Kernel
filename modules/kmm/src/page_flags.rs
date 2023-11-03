// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! ## Page Table Flags
//!
//! Provides page table flags used by [`crate::vmm`] and [`crate::setup`].
//! Page table flags are architecture specific, so this modules has to provide `const`s for each supported architecture.
//! All flags provided allow read/write to the memory, but no execution.
//!
use memory::page_table::PageTableFlags;

/// Page table flags used for intermediate page table to page table transitions and page table to page transitions on the lowest level.
#[cfg(target_arch = "x86_64")]
pub const PAGE_TABLE_FLAGS: PageTableFlags = PageTableFlags::VALID
    .union(PageTableFlags::WRITEABLE)
    .union(PageTableFlags::GLOBAL)
    .union(PageTableFlags::NO_EXECUTE);

/// Page table flags used for page table to page transitions for pages that are bigger than 4096.
#[cfg(target_arch = "x86_64")]
pub const HUGE_PAGE_FLAGS: PageTableFlags = PageTableFlags::VALID
    .union(PageTableFlags::WRITEABLE)
    .union(PageTableFlags::HUGE_PAGE)
    .union(PageTableFlags::GLOBAL)
    .union(PageTableFlags::NO_EXECUTE);
