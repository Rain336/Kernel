//! # Address Extensions Module
//!
//!

use crate::page_table::{PageOffset, PageTableIndex, PageTableLevel};
use common::addr::VirtAddr;

pub trait VirtAddrExt {
    /// Returns the page offset part of this address.
    fn page_offset(self) -> PageOffset;

    /// Returns the 9-bit index for the given [`PageTableLevel`].
    fn page_table_index(self, level: PageTableLevel) -> PageTableIndex;
}

impl VirtAddrExt for VirtAddr {
    fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self.as_u64() as u16)
    }

    fn page_table_index(self, level: PageTableLevel) -> PageTableIndex {
        PageTableIndex::new_truncate((self.as_u64() >> 12 >> ((level.as_u8() - 1) * 9)) as u16)
    }
}
