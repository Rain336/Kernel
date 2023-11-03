// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//! # Address Module
//!
//! A kernel need to work with two kinds of addresses, virtual and physical.
//! A virtual address is the address used by processor for load/store instructions as well as almost all other CPU constructs which access memory.
//! A physical address is the address actually send to the memory chip, meaning a virtual to physical translation has to happen.
//! This translation is done by the memory management unit (MMU), which implies certain restrictions on physical and virtual addresses.
//! For this reason the [`PhysAddr`] and [`VirtAddr`] structs exist and enforce a well formed physical / virtual address.
//! See their respective docs to find out about the implied restrictions.
//!
use crate::memory::get_memory_info;
use core::fmt::{self, Binary, Formatter, LowerHex, Octal, Pointer, UpperHex};
use core::ops::{Add, AddAssign, Sub, SubAssign};

/// Error returned when an invalid address is passed to [`VirtAddr`].
#[derive(Debug)]
pub struct VirtAddrNotValid(pub u64);

/// A virtual memory address
/// Virtual memory addresses are always 64-bit, but segmented into 9-bit indexes, starting after the page offset.
/// How many of these indexes are in used depend on the architecture,
/// with all currently supported architectures (x86_64, AArch64, riscv64) supporting at least 4 levels.
/// All architectures also support an optional 5th level based on feature registers.
/// In addition the the 9-bit indexes, a 12-bit page offset is always placed at the lowest bits of the address.
/// All bits outside the 9-bit indexes and 12-bit page offset are sign extended, meaning they must have the same bits as the highest valid bit.
/// Here an example of a level 5 address:
///
/// |Sign Extend|Level 5|Level 4|Level 3|Level 2|Level 1|Page Offset|
/// |:-:|:-:|:-:|:-:|:-:|:-:|:-:|
/// |1111111|111111000|100011111|111000111|101001101|100110000|000000000000|
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Creates a virtual address from a [`u64`], panics when the address is not well-formed for the current architecture.
    pub fn new(addr: u64) -> Self {
        addr.try_into().expect(
            "Address passed to VirtAddr::new must not contain data in the sign extended bits",
        )
    }

    /// Creates a virtual address from a [`u64`], truncating the sign extend to work on the current architecture.
    pub fn new_truncate(addr: u64) -> Self {
        let bits = 64 - get_memory_info().virtual_address_bits;
        VirtAddr(((addr << bits) as i64 >> bits) as u64)
    }

    /// Creates a virtual address from a [`u64`], truncating the sign extend to work on the lowest common denominator architecture.
    pub const fn new_const(addr: u64) -> Self {
        VirtAddr(((addr << 17) as i64 >> 17) as u64)
    }

    /// Create a virtual address from a [`u64`], with out checking if it's a well formed address for the current architecture.
    ///
    /// ## Safety
    ///
    /// This function does not check if the address is well-formed.
    pub const unsafe fn new_unsafe(addr: u64) -> Self {
        VirtAddr(addr)
    }

    /// Create a virtual address of `0`
    pub const fn zero() -> Self {
        VirtAddr(0)
    }

    /// Converts the virtual address back into a [`u64`]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Creates a pointer out of the virtual address.
    pub const fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Creates a mutable pointer out of the virtual address.
    pub const fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Returns whenever the virtual address is `0`
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Aligns the virtual address up to be a multiple of `align`.
    /// Returns None if the address would overflow.
    pub fn align_up(self, align: u64) -> Option<Self> {
        align_up(self.0, align).map(VirtAddr::new_truncate)
    }

    /// Aligns the virtual address down to be a multiple of `align`.
    pub fn align_down(self, align: u64) -> Self {
        VirtAddr::new_truncate(align_down(self.0, align))
    }

    /// Returns whenever the virtual address is aligned to the give alignment.
    pub fn is_aligned(self, align: u64) -> bool {
        self.align_down(align) == self
    }
}

impl Default for VirtAddr {
    fn default() -> Self {
        VirtAddr::zero()
    }
}

impl From<VirtAddr> for u64 {
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}

impl TryFrom<u64> for VirtAddr {
    type Error = VirtAddrNotValid;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let bits = get_memory_info().virtual_address_bits - 1;
        let mask = u64::MAX << bits;
        match (value & mask) >> bits {
            0 => Ok(VirtAddr(value)),
            1 => Ok(VirtAddr::new_truncate(value)),
            x => {
                if x == (mask >> bits) {
                    Ok(VirtAddr(value))
                } else {
                    Err(VirtAddrNotValid(value))
                }
            }
        }
    }
}

impl<T> From<*const T> for VirtAddr {
    fn from(value: *const T) -> Self {
        VirtAddr::new_truncate(value as u64)
    }
}

impl<T> From<*mut T> for VirtAddr {
    fn from(value: *mut T) -> Self {
        VirtAddr::new_truncate(value as u64)
    }
}

impl Add<VirtAddr> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr::new(self.0 + rhs.0)
    }
}

impl Add<u64> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 + rhs)
    }
}

impl Add<usize> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, rhs: usize) -> Self::Output {
        VirtAddr::new(self.0 + rhs as u64)
    }
}

impl AddAssign<VirtAddr> for VirtAddr {
    fn add_assign(&mut self, rhs: VirtAddr) {
        *self = *self + rhs;
    }
}

impl AddAssign<u64> for VirtAddr {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Binary for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Binary::fmt(&self.0, f)
    }
}

impl LowerHex for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl Octal for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Octal::fmt(&self.0, f)
    }
}

impl Pointer for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: VirtAddr) -> Self::Output {
        VirtAddr::new(self.0 - rhs.0)
    }
}

impl Sub<u64> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::new(self.0 - rhs)
    }
}

impl Sub<usize> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        VirtAddr::new(self.0 - rhs as u64)
    }
}

impl SubAssign<VirtAddr> for VirtAddr {
    fn sub_assign(&mut self, rhs: VirtAddr) {
        *self = *self - rhs
    }
}

impl SubAssign<u64> for VirtAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs
    }
}

impl SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs
    }
}

impl UpperHex for VirtAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

/// Error returned when an invalid address is passed to [`PhysAddr`].
#[derive(Debug)]
pub struct PhysAddrNotValid(pub u64);

/// A physical memory address
/// Physical address are always 64 bits long, but not all bits are in use.
/// How many bits are in use is architecture and mmu dependant, but all currently supported architectures at least supporting 52-bits.
/// All unused bits have to be zero.
/// An example of a 52-bit address:
///
/// |Unused|Physical Address|
/// |:-:|:-:|
/// |000000000000|1111000000000011100000000110000011100000000011000100|
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(u64);

impl PhysAddr {
    /// Creates a physical address from a [`u64`], panics when the address is not well-formed for the current architecture.
    pub fn new(addr: u64) -> Self {
        addr.try_into()
            .expect("Address passed to PhysAddr::new must not contain data in the ignored bits")
    }

    /// Creates a physical address forma [`u64`], truncating the unused start of the address to work on the current architecture.
    pub fn new_truncate(addr: u64) -> Self {
        let max = 1 << get_memory_info().physical_address_bits;
        PhysAddr(addr % max)
    }

    /// Create a physical address from a [`u64`], with out checking if it's a well-formed address for the current architecture.
    ///
    /// ## Safety
    ///
    /// This function does not check if the address is well-formed.
    pub const unsafe fn new_unsafe(addr: u64) -> Self {
        PhysAddr(addr)
    }

    /// Create a physical address of `0`
    pub const fn zero() -> Self {
        PhysAddr(0)
    }

    /// Converts the physical address back into a [`u64`]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns whenever the physical address is `0`
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Aligns the physical address down to be a multiple of `align`.
    /// Returns None if the address would overflow.
    pub fn align_up(self, align: u64) -> Option<Self> {
        align_up(self.0, align).map(PhysAddr::new_truncate)
    }

    /// Aligns the physical address down to be a multiple of `align`.
    pub fn align_down(self, align: u64) -> Self {
        PhysAddr::new_truncate(align_down(self.0, align))
    }

    /// Returns whenever the physical address is aligned to the give alignment.
    pub fn is_aligned(self, align: u64) -> bool {
        self.align_down(align) == self
    }
}

impl Default for PhysAddr {
    fn default() -> Self {
        PhysAddr::zero()
    }
}

impl TryFrom<u64> for PhysAddr {
    type Error = PhysAddrNotValid;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        let truncated = PhysAddr::new_truncate(value);
        if truncated.0 == value {
            Ok(truncated)
        } else {
            Err(PhysAddrNotValid(value))
        }
    }
}

impl From<PhysAddr> for u64 {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

impl Add<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr::new(self.0 + rhs.0)
    }
}

impl Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0 + rhs)
    }
}

impl Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        PhysAddr::new(self.0 + rhs as u64)
    }
}

impl AddAssign<PhysAddr> for PhysAddr {
    fn add_assign(&mut self, rhs: PhysAddr) {
        *self = *self + rhs
    }
}

impl AddAssign<u64> for PhysAddr {
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs
    }
}

impl AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs
    }
}

impl Binary for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Binary::fmt(&self.0, f)
    }
}

impl LowerHex for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl Octal for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Octal::fmt(&self.0, f)
    }
}

impl Pointer for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        PhysAddr::new(self.0 - rhs.0)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0 - rhs)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        PhysAddr::new(self.0 - rhs as u64)
    }
}

impl SubAssign<PhysAddr> for PhysAddr {
    fn sub_assign(&mut self, rhs: PhysAddr) {
        *self = *self - rhs
    }
}

impl SubAssign<u64> for PhysAddr {
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs
    }
}

impl SubAssign<usize> for PhysAddr {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs
    }
}

impl UpperHex for PhysAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

const fn align_up(addr: u64, align: u64) -> Option<u64> {
    assert!(align.is_power_of_two(), "Align has to be a power of two");
    let mask = align - 1;
    if (addr & mask) == 0 {
        Some(addr)
    } else {
        (addr | mask).checked_add(1)
    }
}

const fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "Align has to be a power of two");
    addr & !(align - 1)
}
