// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Frame module
//!
//! A frame is a mappable block of physical memory.
//! To see which sizes exist and which are supported by which architecture, see [`PageSize`].
//! Mapping works by translating a [`super::page::Page`] to a [`PhysFrame`] of the same size.
//! Frames have to be aligned to it's size, to allow the whole physical memory to be divided into frames.
//!
use super::size::{PageSize, Size4KiB};
use super::AddressNotAligned;
use common::addr::PhysAddr;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A block of `S::SIZE` bytes physically addressed memory, aligned to it's size.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysFrame<S: PageSize = Size4KiB> {
    start_address: PhysAddr,
    size: PhantomData<S>,
}

impl<S: PageSize> PhysFrame<S> {
    /// The size of the frame in bytes.
    pub const SIZE: u64 = S::SIZE;

    /// Creates a frame from a starting physical address, returns `Ok(PhysFrame)` if the starting address is aligned, `Err(AddressNotAligned)` otherwise.
    pub fn from_start_address(address: PhysAddr) -> Result<Self, AddressNotAligned> {
        if !address.is_aligned(S::SIZE) {
            Err(AddressNotAligned)
        } else {
            Ok(PhysFrame {
                start_address: address,
                size: PhantomData,
            })
        }
    }

    /// Creates a frame from a physical starting address, without checking if the address is correctly aligned.
    ///
    /// ## Safety
    ///
    /// This function does not check if the address is correctly aligned.
    pub const unsafe fn from_start_address_unchecked(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address,
            size: PhantomData,
        }
    }

    /// Returns the frame which contains the given physical address.
    pub fn containing_address(address: PhysAddr) -> Self {
        PhysFrame {
            start_address: address.align_down(S::SIZE),
            size: PhantomData,
        }
    }

    /// Returns the starting address of the frame.
    pub const fn start_address(self) -> PhysAddr {
        self.start_address
    }

    /// Creates a range of frames from start up to but not including end.
    pub const fn range(start: PhysFrame<S>, end: PhysFrame<S>) -> PhysFrameRange<S> {
        PhysFrameRange { start, end }
    }

    /// Creates a range of frames from start up to and including end.
    pub const fn range_inclusive(
        start: PhysFrame<S>,
        end: PhysFrame<S>,
    ) -> PhysFrameRangeInclusive<S> {
        PhysFrameRangeInclusive { start, end }
    }
}

impl<S: PageSize> Add<u64> for PhysFrame<S> {
    type Output = PhysFrame<S>;

    fn add(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address + rhs * S::SIZE)
    }
}

impl<S: PageSize> AddAssign<u64> for PhysFrame<S> {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs
    }
}

impl<S: PageSize> Sub<u64> for PhysFrame<S> {
    type Output = PhysFrame<S>;

    fn sub(self, rhs: u64) -> Self::Output {
        PhysFrame::containing_address(self.start_address - rhs * S::SIZE)
    }
}

impl<S: PageSize> SubAssign<u64> for PhysFrame<S> {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs
    }
}

impl<S: PageSize> Sub<PhysFrame<S>> for PhysFrame<S> {
    type Output = u64;

    fn sub(self, rhs: PhysFrame<S>) -> Self::Output {
        (self.start_address - rhs.start_address).as_u64() / S::SIZE
    }
}

/// A non-inclusive range of frames from start to end.
/// An empty or negative range always returns `None`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysFrameRange<S: PageSize = Size4KiB> {
    /// the first frame in the range
    pub start: PhysFrame<S>,
    /// the last frame in the range
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRange<S> {
    /// Returns whenever the range is empty.
    pub const fn is_empty(self) -> bool {
        // NOTE: const traits would make this a lot simpler
        self.start.start_address.as_u64() >= self.end.start_address.as_u64()
    }
}

impl<S: PageSize> Iterator for PhysFrameRange<S> {
    type Item = PhysFrame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let frame = self.start;
            self.start += 1;
            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> ExactSizeIterator for PhysFrameRange<S> {
    fn len(&self) -> usize {
        if self.start < self.end {
            (self.end - self.start) as usize
        } else {
            0
        }
    }
}

impl<S: PageSize> FusedIterator for PhysFrameRange<S> {}

/// A inclusive range of frames from start to end.
/// An empty or negative range always returns `None`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysFrameRangeInclusive<S: PageSize = Size4KiB> {
    /// the first frame in the range
    pub start: PhysFrame<S>,
    /// the last frame in the range
    pub end: PhysFrame<S>,
}

impl<S: PageSize> PhysFrameRangeInclusive<S> {
    /// Returns whenever the range is empty.
    pub const fn is_empty(self) -> bool {
        // NOTE: const traits would make this a lot simpler
        self.start.start_address.as_u64() > self.end.start_address.as_u64()
    }
}

impl<S: PageSize> Iterator for PhysFrameRangeInclusive<S> {
    type Item = PhysFrame<S>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start;

            let max_frame_address = PhysAddr::new_truncate(u64::MAX) - (S::SIZE - 1);
            if self.start.start_address < max_frame_address {
                self.start += 1;
            } else {
                self.end -= 1;
            }

            Some(frame)
        } else {
            None
        }
    }
}

impl<S: PageSize> ExactSizeIterator for PhysFrameRangeInclusive<S> {
    fn len(&self) -> usize {
        if self.start <= self.end {
            (self.end - self.start) as usize + 1
        } else {
            0
        }
    }
}

impl<S: PageSize> FusedIterator for PhysFrameRangeInclusive<S> {}
