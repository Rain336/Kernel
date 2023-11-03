//! # Module Interface
//!
//! This create is depended on by all other kernel modules to interface with information supplied by the bootloader and also other modules.
//! It is passed to the module's entrypoint, which in turn gets wired up using module runner.
//!
#![no_std]

/// Interface to be used bt the different kernel modules.
pub struct ModuleInterface {
    /// Provides info about the kernel's stacks.
    pub stack_info: StackInfo,

    /// Pointer to the Root System Description Pointer (RSDP) or `0` if this system doesn't have ACPI.
    pub rsdp_address: u64,

    /// Provides a framebuffer to draw into or `None` if no framebuffer could be acquired.
    pub framebuffer_info: Option<FramebufferInfo>,

    /// Provides a memory map.
    pub memory_map_info: MemoryMapInfo,

    /// Provides info about the MMU.
    pub memory_info: MemoryInfo,
}

/// Provides info about the kernel's stack.
pub struct StackInfo {
    /// Pointer to the kernel's primary stack.
    pub primary_stack: u64,

    /// Length to the kernel's primary stack, in bytes.
    pub primary_stack_len: usize,

    /// Pointer to the kernel's secondary stack.
    pub secondary_stack: u64,

    /// Length to the kernel's secondary stack, in bytes.
    pub secondary_stack_len: usize,
}

/// Provides info about a framebuffer.
/// A color is always 32 bits and each color component is 8 bits.
pub struct FramebufferInfo {
    /// The start of the memory-mapped framebuffer.
    pub address: *mut u8,

    /// The size of the frambuffer in bytes.
    pub size: usize,

    /// The width of the frmebuffer screen.
    pub width: u64,

    /// The height of the frmebuffer screen.
    pub height: u64,

    /// The about of bytes that make up one row.
    pub pitch: u64,

    /// Amount to shift the 8-bit red color part by.
    pub red_mask_shift: u8,

    /// Amount to shift the 8-bit green color part by.
    pub green_mask_shift: u8,

    /// Amount to shift the 8-bit blue color part by.
    pub blue_mask_shift: u8,
}

/// Provides a memory map.
pub struct MemoryMapInfo {
    /// Pointer to the memory map buffer.
    pub memory_map: u64,

    /// The number of entries in the memory map.
    pub memory_map_count: usize,

    /// Type of the memory map.
    pub memory_map_type: MemoryMapType,
}

/// The type of the memory map.
/// Describes it's entries layout and values.
pub enum MemoryMapType {
    /// The memory map is an array of Limine MemmapEntry structs.
    Limine,
}

/// Information about the MMU of this system.
pub struct MemoryInfo {
    /// How many bits a virtual address can have.
    pub virtual_address_bits: u64,

    /// How many bits a physical address can have.
    pub physical_address_bits: u64,

    /// Mask to extract the address from a page table entry.
    pub page_table_entry_address_mask: u64,

    /// The highest level of page table supported.
    pub highest_page_table_level: u8,
}
