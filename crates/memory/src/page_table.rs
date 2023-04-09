//! # Page Table Module
//!
//! Page tables describe how the translation from virtual to physical addresses is done.
//! A page table consists of 512 entries with each entry either pointing to another page table or defining a mapping.
//! Page tables form a tree-like structure with the root page table pointing to lower levels.
//! For the actual translation, the address is split into indexes as seen on [`super::addr::VirtAddr`].
//! These indexes are used to index into the page tables, with the highest level being the root table and going lower from there.
//! If a table terminates early, the remaning indexes as well as the page offset are directly mapped to the physical address.
use super::size::{PageSize, Size4KiB};
use bitflags::bitflags;
use common::addr::PhysAddr;
use common::memory::MEMORY_INFO;
use common::sync::CriticalSection;
use core::ops::{Index, IndexMut};
use core::sync::atomic::{AtomicU64, Ordering};

/// 12-bit page offset at the begining of a virtual address.
/// It is always dirrectly mapped to a physical address, since 4 kibibytes are the smallest mappable unit.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageOffset(u16);

impl PageOffset {
    /// Creates a new page offset, panics if value >= 4096.
    pub const fn new(value: u16) -> Self {
        assert!(value < 4096);
        PageOffset(value)
    }

    /// Creates a new page offset, truncating it to 4096.
    /// `value % 4096`
    pub const fn new_truncate(value: u16) -> Self {
        PageOffset(value % 4096)
    }
}

impl From<PageOffset> for u16 {
    fn from(value: PageOffset) -> Self {
        value.0
    }
}

impl From<PageOffset> for u32 {
    fn from(value: PageOffset) -> Self {
        value.0 as u32
    }
}

impl From<PageOffset> for u64 {
    fn from(value: PageOffset) -> Self {
        value.0 as u64
    }
}

impl From<PageOffset> for usize {
    fn from(value: PageOffset) -> Self {
        value.0 as usize
    }
}

/// An 9-bit index into a page table.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Creates a page table index, panics if index >= 512
    pub const fn new(index: u16) -> Self {
        assert!(index < 512);
        PageTableIndex(index)
    }

    /// Creates a page table index, truncating it to 512.
    /// `value % 512`
    pub const fn new_truncate(index: u16) -> Self {
        PageTableIndex(index % 512)
    }
}

impl From<PageTableIndex> for u16 {
    fn from(value: PageTableIndex) -> Self {
        value.0
    }
}

impl From<PageTableIndex> for u32 {
    fn from(value: PageTableIndex) -> Self {
        value.0 as u32
    }
}

impl From<PageTableIndex> for u64 {
    fn from(value: PageTableIndex) -> Self {
        value.0 as u64
    }
}

impl From<PageTableIndex> for usize {
    fn from(value: PageTableIndex) -> Self {
        value.0 as usize
    }
}

/// A page table level
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageTableLevel(u8);

impl PageTableLevel {
    /// Create a page table level from an [`u8`].
    /// Panics if the level is unsupported.
    pub fn new(value: u8) -> Self {
        assert!(value <= MEMORY_INFO.highest_page_table_level);
        PageTableLevel(value)
    }

    /// Create a page table level from an [`u8`], without checking if it's supported.
    ///
    /// ## Safety
    ///
    /// This function allows creating page table levels wich might not be supported.
    pub const unsafe fn new_unsafe(value: u8) -> Self {
        PageTableLevel(value)
    }

    /// Returns the highest page table level supported.
    pub fn highest() -> Self {
        PageTableLevel(MEMORY_INFO.highest_page_table_level)
    }

    /// Returns the current page table level as a [`u8`].
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Get the next lower level or `None` if it's the lowest.
    pub const fn next_lower_level(self) -> Option<Self> {
        match self.0.checked_sub(1) {
            None | Some(0) => None,
            Some(x) => Some(PageTableLevel(x)),
        }
    }

    pub const fn is_last(self) -> bool {
        self.0 == 1
    }

    pub const fn address_space_mask(self) -> u64 {
        (1 << ((self.0 - 1) * 9 + 12)) - 1
    }

    pub const fn page_table_mask(self) -> u64 {
        (1 << (self.0 * 9 + 12)) - 1
    }
}

impl TryFrom<u8> for PageTableLevel {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > MEMORY_INFO.highest_page_table_level {
            Err(())
        } else {
            Ok(PageTableLevel(value))
        }
    }
}

impl From<PageTableLevel> for u8 {
    fn from(value: PageTableLevel) -> Self {
        value.0
    }
}

#[cfg(target_arch = "x86_64")]
bitflags! {
    /// Flags used to control a page table entry.
    pub struct PageTableFlags: u64 {
        /// Must be set for the entry to be vaild, otherwise the entry is ignored.
        const VALID = 1;

        /// Allows writing into the page.
        /// Higher levels take precedance over lower levels.
        const WRITEABLE = 1 << 1;

        /// Allows accessing the page from rings > 0.
        /// Higher levels take precedance over lower levels.
        const USER_ACCESSABLE = 1 << 2;

        const WRITE_THROUGH = 1 << 3;

        /// Disables caching for the mapped region.
        const NO_CACHE = 1 << 4;

        /// Set by the processor when a read happens in the mapped region.
        const ACCESSED = 1 << 5;

        /// Set by the processor when a write happens in the mapped region.
        const DIRTY = 1 << 6;

        /// Says that this entry doesn't point to another page table, but rather directly to a mapped region.
        const HUGE_PAGE = 1 << 7;

        /// Keeps the mapping in cache, even when the addrass space gets swapped.
        const GLOBAL = 1 << 8;

        const BIT_9 = 1 << 9;
        const BIT_10 = 1 << 10;
        const BIT_11 = 1 << 11;
        const BIT_52 = 1 << 52;
        const BIT_53 = 1 << 53;
        const BIT_54 = 1 << 54;
        const BIT_55 = 1 << 55;
        const BIT_56 = 1 << 56;
        const BIT_57 = 1 << 57;
        const BIT_58 = 1 << 58;
        const BIT_59 = 1 << 59;
        const BIT_60 = 1 << 60;
        const BIT_61 = 1 << 61;
        const BIT_62 = 1 << 62;

        /// Prevents execution in the mapped region.
        const NO_EXECUTE = 1 << 63;
    }
}

#[cfg(riscv)]
bitflags! {
    /// Flags used to control a page table entry.
    ///
    /// The READ, WRITE and EXECUTE bits define if entry points to another table or a block of memory to be mapped.
    /// If READ, WRITE and EXECUTE are all unset, this entry points to another table otherwise, mapping terminates here.
    /// Also, WRITE requires READ to be set, WRITE and WRITE + EXECUTE are not vaild and reserved for future use.
    pub struct PageTableFlags: u64 {
        /// Must be set for the entry to be vaild, otherwise the entry is ignored.
        const VALID = 1;

        /// Allows reading inside the mapped region.
        const READ = 1 << 1;

        /// Allows writing in the mapped region.
        const WRITE = 1 << 2;

        /// Allows execution in the mapped region.
        const EXECUTE = 1 << 3;

        /// Allows access to the mapped region from user mode.
        const USER_ACCESSABLE = 1 << 4;

        /// Keeps the mapping in cache, even when the addrass space gets swapped.
        const GLOBAL = 1 << 5;

        /// Set by the processor when a read happens in the mapped region.
        const ACCESSED = 1 << 6;

        /// Set by the processor when a write happens in the mapped region.
        const DIRTY = 1 << 7;

        const BIT_8 = 1 << 8;
        const BIT_9 = 1 << 9;
    }
}

// bitflags! {
//     pub struct PageTableFlags: u64 {
//         const VALID = 1;
//         const TABLE_DESCRIPTOR = 1 << 1;
//     }
// }

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageTranslation {
    PageTable(PhysAddr),
    Page(PhysAddr),
    None,
}

/// An entry in a page table.
/// Page table entries are 64 bits on all currently supported platforms (x86_64, AArch64, resicv64).
/// The make up of these is architecture specific with each haveing diffrent [`PageTableFlags`].
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Creates a new page table entry.
    pub fn new(addr: PhysAddr, flags: PageTableFlags) -> Self {
        assert!(addr.is_aligned(Size4KiB::SIZE));
        PageTableEntry(addr.as_u64() | flags.bits())
    }

    /// Creates a new unused page table entry.
    pub const fn unused() -> Self {
        PageTableEntry(0)
    }

    /// Returns whenever the entry is unused.
    pub const fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Sets the entry as unused.
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Returns the [`PageTableFlags`] of this entry.
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.0)
    }

    /// Returns the [`PhysAddr`] of this entry.
    /// This can be a pointer to the next page table or a pointer to a block of memory to be mapped.
    /// Wich of these depends on the [`PageTableFlags`] of this entry.
    pub fn addr(&self) -> PhysAddr {
        let mask = MEMORY_INFO.page_table_entry_address_mask;
        PhysAddr::new(self.0 & mask)
    }

    /// Sets the [`PhysAddr`] and [`PageTableFlags`] of this entry.
    pub fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        assert!(addr.is_aligned(Size4KiB::SIZE));
        self.0 = addr.as_u64() | flags.bits();
    }

    /// Sets only the [`PageTableFlags`] of this entry.
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.0 = self.addr().as_u64() | flags.bits();
    }

    #[cfg(target_arch = "x86_64")]
    pub fn translate(&self) -> PageTranslation {
        let flags = self.flags();
        if !flags.contains(PageTableFlags::VALID) {
            return PageTranslation::None;
        }

        if flags.contains(PageTableFlags::HUGE_PAGE) {
            return PageTranslation::Page(self.addr());
        }

        PageTranslation::PageTable(self.addr())
    }

    #[cfg(riscv)]
    pub fn translate(&self) -> PageTranslation {
        let flags = self.flags();
        if !flags.contains(PageTableFlags::VALID) {
            return PageTranslation::None;
        }

        if flags.intersects(
            PageTableFlags::READ
                .union(PageTableFlags::WRITE)
                .union(PageTableFlags::EXECUTE),
        ) {
            return PageTranslation::Page(self.addr());
        }

        PageTranslation::PageTable(self.addr())
    }
}

#[cfg(target_arch = "x86_64")]
pub const LOCK_BIT: PageTableFlags = PageTableFlags::BIT_10;
#[cfg(riscv)]
pub const LOCK_BIT: PageTableFlags = PageTableFlags::BIT_9;

const LOCK_BIT_U64: u64 = LOCK_BIT.bits;

/// A [`PageTableEntry`] that allows concurrent access.
/// This is doen by defining one bit as a [`LOCK_BIT`] that when set,
/// signals the entry is being initilized.
/// Currently entries cannot be deleted, but since this technique is only needed for kernal memory mapping,
/// it is sufficent to just leave potental dead pages linger.
#[repr(transparent)]
pub struct LockedPageTableEntry(AtomicU64);

impl LockedPageTableEntry {
    /// Creates an unused, uninitilized entry.
    pub const fn new() -> Self {
        LockedPageTableEntry(AtomicU64::new(0))
    }

    /// Reads the page table entry, if it's initilized
    pub fn get(&self) -> Option<PageTableEntry> {
        let current = PageTableEntry(self.0.load(Ordering::Relaxed));
        if current.flags().contains(PageTableFlags::VALID) {
            return Some(current);
        }

        let mut current = PageTableEntry(self.0.load(Ordering::Acquire));
        while current.flags().contains(LOCK_BIT) {
            current = PageTableEntry(self.0.load(Ordering::Acquire));
        }

        if current.flags().contains(PageTableFlags::VALID) {
            Some(current)
        } else {
            None
        }
    }

    /// Initializes the entry to the given value.
    /// Returns `Ok` if the value could be initilized, `Err` if the vaile was already initilized.
    pub fn set(&self, value: PageTableEntry) -> Result<(), PageTableEntry> {
        let _section = CriticalSection::new();
        match self
            .0
            .compare_exchange(0, LOCK_BIT_U64, Ordering::AcqRel, Ordering::Relaxed)
        {
            Ok(_) => {
                self.0.store(value.0, Ordering::Release);
                Ok(())
            }
            Err(_) => Err(value),
        }
    }

    /// Initilizes the entry using the given function or returns the already initilized value.
    pub fn get_or_init(&self, f: impl FnOnce() -> PageTableEntry) -> PageTableEntry {
        let section = CriticalSection::new();
        match self
            .0
            .compare_exchange(0, LOCK_BIT_U64, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {
                let value = f();
                self.0.store(value.0, Ordering::Release);
                value
            }
            Err(value) => {
                drop(section);

                let mut value = PageTableEntry(value);
                while value.flags().contains(LOCK_BIT) {
                    value = PageTableEntry(self.0.load(Ordering::Acquire));
                }

                value
            }
        }
    }

    /// Sets the entry as unused.
    pub fn set_unused(&self) -> PageTableEntry {
        PageTableEntry(self.0.swap(0, Ordering::AcqRel))
    }
}

/// A page table used for mapping.
#[derive(Clone)]
#[repr(C, align(4096))]
pub struct PageTable {
    table: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> Self {
        PageTable {
            table: [PageTableEntry::unused(); 512],
        }
    }

    pub fn zero(&mut self) {
        for i in self.table.iter_mut() {
            i.set_unused();
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, PageTableEntry> {
        self.table.iter()
    }

    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, PageTableEntry> {
        self.table.iter_mut()
    }
}

impl Default for PageTable {
    fn default() -> Self {
        PageTable::new()
    }
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.table[index.0 as usize]
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.table[index.0 as usize]
    }
}

impl IndexMut<usize> for PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.table[index]
    }
}

#[repr(C, align(4096))]
pub struct LockedPageTable {
    table: [LockedPageTableEntry; 512],
}

impl LockedPageTable {
    pub const fn new() -> Self {
        // This constant is not accessable outside the new function and shouldn't be
        // It is only used to initialize the locked page table.
        #[allow(clippy::declare_interior_mutable_const)]
        const EMPTY: LockedPageTableEntry = LockedPageTableEntry::new();

        LockedPageTable {
            table: [EMPTY; 512],
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, LockedPageTableEntry> {
        self.table.iter()
    }
}

impl Default for LockedPageTable {
    fn default() -> Self {
        LockedPageTable::new()
    }
}

impl Index<PageTableIndex> for LockedPageTable {
    type Output = LockedPageTableEntry;

    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.table[index.0 as usize]
    }
}

impl Index<usize> for LockedPageTable {
    type Output = LockedPageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}
