//! # Page Module
//!
//! A page is a mappable block of virtual memory.
//! To see wich sized exist and wich are supported by wich archetecture, see [`PageSize`].
//! Mapping works by translating a [`Page`] to a [`super::frame::PhysFrame`] of the same size.
//! Pages have to be alligned to it's size, to allow the whole virtual memory to be devided into pages.
use super::page_table::{PageTableIndex, PageTableLevel};
use super::size::{PageSize, Size4KiB};
use super::AddressNotAligned;
use common::addr::VirtAddr;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A block of `S::SIZE` bytes virtually addressed memory, aligned to it's size.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Page<S: PageSize = Size4KiB> {
    start_address: VirtAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    /// The size of the page in bytes.
    pub const SIZE: u64 = S::SIZE;

    /// Creates a page from the given virtual address, returns `Err(AddressNotAligned)` if the address is not correctly aliged to it's size.
    pub fn from_start_address(address: VirtAddr) -> Result<Self, AddressNotAligned> {
        if !address.is_aligned(S::SIZE) {
            Err(AddressNotAligned)
        } else {
            Ok(Page {
                start_address: address,
                size: PhantomData,
            })
        }
    }

    /// Creates a page from the given virtual address, without checking if it's correctly aligned.
    ///
    /// ## Safety
    ///
    /// This function does not check if the address is correctly aligned.
    pub const unsafe fn from_start_address_unchecked(address: VirtAddr) -> Self {
        Page {
            start_address: address,
            size: PhantomData,
        }
    }

    /// Returns the page wich contains the given virtual address.
    pub fn containing_address(address: VirtAddr) -> Self {
        Page {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the starting address of this page.
    pub const fn start_address(self) -> VirtAddr {
        self.start_address
    }

    /// Returns the 9-bit index for the given [`PageTableLevel`].
    pub const fn page_table_index(self, level: PageTableLevel) -> PageTableIndex {
        // should be just self.start_address.page_table_index(level)
        PageTableIndex::new_truncate(
            (self.start_address.as_u64() >> 12 >> ((level.as_u8() - 1) * 9)) as u16,
        )
    }

    /// Creates a range of pages from start up to but not including end.
    pub const fn range(start: Page<S>, end: Page<S>) -> PageRange<S> {
        PageRange { start, end }
    }

    /// Creates a range of pages from start up to and including end.
    pub const fn range_inclusive(start: Page<S>, end: Page<S>) -> PageRangeInclusive<S> {
        PageRangeInclusive { start, end }
    }
}

impl<S: PageSize> Add<u64> for Page<S> {
    type Output = Page<S>;

    fn add(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for Page<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs
    }
}

impl<S: PageSize> Sub<u64> for Page<S> {
    type Output = Page<S>;

    fn sub(self, rhs: u64) -> Self::Output {
        Page::containing_address(self.start_address - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for Page<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs
    }
}

impl<S: PageSize> Sub<Page<S>> for Page<S> {
    type Output = u64;

    fn sub(self, rhs: Page<S>) -> Self::Output {
        (self.start_address - rhs.start_address).as_u64() / S::SIZE
    }
}

/// A non-inclusive range of pages from start to end.
/// An empty or negative range always returns `None`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageRange<S: PageSize> {
    /// the first page in the range
    pub start: Page<S>,
    /// the last page in the range
    pub end: Page<S>,
}

impl<S: PageSize> PageRange<S> {
    /// Returns whenever the range is empty.
    pub const fn is_empty(self) -> bool {
        self.start.start_address.as_u64() >= self.end.start_address.as_u64()
    }
}

impl<S: PageSize> Iterator for PageRange<S> {
    type Item = Page<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let page = self.start;
            self.start += 1;
            Some(page)
        } else {
            None
        }
    }
}

impl<S: PageSize> ExactSizeIterator for PageRange<S> {
    fn len(&self) -> usize {
        if self.start < self.end {
            (self.end - self.start) as usize
        } else {
            0
        }
    }
}

impl<S: PageSize> FusedIterator for PageRange<S> {}

/// A inclusive range of pages from start to end.
/// An empty or negative range always returns `None`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageRangeInclusive<S: PageSize> {
    /// the first page in the range
    pub start: Page<S>,
    /// the last page in the range
    pub end: Page<S>,
}

impl<S: PageSize> PageRangeInclusive<S> {
    /// Returns whenever the range is empty.
    pub const fn is_empty(self) -> bool {
        self.start.start_address.as_u64() > self.end.start_address.as_u64()
    }
}

impl<S: PageSize> Iterator for PageRangeInclusive<S> {
    type Item = Page<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let page = self.start;

            let max_page_address = VirtAddr::new_truncate(u64::MAX) - (S::SIZE - 1);
            if self.start.start_address < max_page_address {
                self.start += 1;
            } else {
                self.end -= 1;
            }

            Some(page)
        } else {
            None
        }
    }
}

impl<S: PageSize> ExactSizeIterator for PageRangeInclusive<S> {
    fn len(&self) -> usize {
        if self.start <= self.end {
            (self.end - self.start) as usize + 1
        } else {
            0
        }
    }
}

impl<S: PageSize> FusedIterator for PageRangeInclusive<S> {}
