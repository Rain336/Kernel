/// The memory info struct tells the kernel about what regions of memory are freely useable and which ones are in use and by what.
/// The main [`MemoryInfo`] struct is just an array of [`MemoryDescriptor`]s.
#[repr(C)]
pub struct MemoryInfo {
    /// The length of the `descriptors` array, in number of [`MemoryDescriptor`] structs.
    pub count: u64,

    /// An array of [`MemoryDescriptor`]s, describing the avilable memory.
    pub descriptors: *mut MemoryDescriptor,
}

impl AsRef<[MemoryDescriptor]> for MemoryInfo {
    fn as_ref(&self) -> &[MemoryDescriptor] {
        unsafe { core::slice::from_raw_parts(self.descriptors, self.count as usize) }
    }
}

impl AsMut<[MemoryDescriptor]> for MemoryInfo {
    fn as_mut(&mut self) -> &mut [MemoryDescriptor] {
        unsafe { core::slice::from_raw_parts_mut(self.descriptors, self.count as usize) }
    }
}

/// A [`MemoryDescriptor`] defins a section of memory and says if it's available or what it is used by.
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type is an enum that describes the availablility of this section.
    pub ty: MemoryType,

    /// The physical starting address of this section.
    pub start_address: u64,

    /// The physical ending address of this section.
    pub end_address: u64,
}

impl MemoryDescriptor {
    /// Gets the length of this section in bytes.
    pub fn len(&self) -> u64 {
        self.end_address - self.start_address
    }

    /// Returns whenever this section is empty.
    pub fn is_empty(&self) -> bool {
        self.end_address == self.start_address
    }
}

/// [`MemoryType`] is an enum that describes the availablility of this section.
#[derive(PartialEq, Eq)]
#[repr(u64)]
pub enum MemoryType {
    /// Reserved memory is used in some form.
    /// Either through MMIO or the kernel.
    Reserved = 0,

    /// Unuseable is memory where errors have been detected and consiquently shouldn't be written to.
    Unuseable = 1,

    /// ACPI memory contins ACPI tables that can be recalimed after the tables have been read out.
    Acpi = 2,

    /// ACPI non volatile memory has to be saved during power state changes.
    AcpiNonVolatile = 3,

    /// Available memory can be freely used by the kernel.
    Available = 4,
}

#[cfg(debug_assertions)]
extern "C" fn _assert_ffi(_: MemoryDescriptor) {}
