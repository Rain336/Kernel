#![no_std]
#![no_main]
#![feature(abi_efiapi)]

extern crate alloc;

mod loader;
mod mapper;
mod sdt;

use alloc::boxed::Box;
use bootloader::{
    BootInfo, KernelPosition, MemoryDescriptor, MemoryInfo, CURRENT_REVISION, KERNEL_PATH,
};
use core::{mem, slice};
use log::{debug, info};
use mapper::KERNEL_CODE_START;
use uefi::prelude::{entry, BootServices};
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode};
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::table::{Boot, SystemTable};
use uefi::{CString16, Handle, Result, ResultExt, Status};

fn allocate_memory_map(boot: &BootServices) -> Result<&'static mut [u8]> {
    let size = boot.memory_map_size();
    let size = size.map_size + 2 * size.entry_size;
    let pages = (size + 4095) >> 12;

    let buffer = unsafe {
        let ptr = boot.allocate_pages(
            AllocateType::AnyPages,
            MemoryType::BOOT_SERVICES_DATA,
            pages,
        )? as *mut u8;
        slice::from_raw_parts_mut(ptr, pages * 4096)
    };
    buffer.fill(0);

    Ok(buffer)
}

fn load_file_from_disk(
    boot: &BootServices,
    image: Handle,
    path: &str,
) -> Result<&'static mut [u8]> {
    debug!("Loading file {} from EFI Partition...", path);
    let path = path.replace('/', "\\");
    let path = CString16::try_from(path.as_str()).unwrap();

    let mut protocol = boot.get_image_file_system(image)?;
    let mut volume = protocol.open_volume()?;
    let handle = volume.open(&path, FileMode::Read, FileAttribute::empty())?;
    let mut file = match handle.into_regular_file() {
        Some(x) => x,
        None => return Err(Status::NOT_FOUND.into()),
    };

    let info = file.get_boxed_info::<FileInfo>()?;

    let ptr = boot.allocate_pool(MemoryType::BOOT_SERVICES_DATA, info.file_size() as usize)?;
    let buffer = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, info.file_size() as usize) };
    buffer.fill(0);

    file.read(buffer).discard_errdata()?;

    Ok(buffer)
}

#[entry]
fn main(image: Handle, mut st: SystemTable<Boot>) -> Status {
    match uefi_services::init(&mut st).status() {
        Status::SUCCESS => {}
        x => return x,
    };

    info!("Loading kernel from EFI System Partition...");
    let kernel = load_file_from_disk(st.boot_services(), image, KERNEL_PATH).unwrap();
    let mut mapper = mapper::MemoryMapper::new();

    info!("Mapping Kernel into memory...");
    let (entry, tls) = loader::load_kernel(kernel, &mut mapper, st.boot_services()).unwrap();

    info!("Reading ACPI Information...");
    let sdt_address = sdt::find_sdt_address(st.config_table());

    // TODO: Map into kernel memory region.
    info!("Allocating Kernel stack memory...");
    let stack_start = KERNEL_CODE_START - 8 * 4096;
    let stack_end = KERNEL_CODE_START;
    let stack = st
        .boot_services()
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 8)
        .unwrap();
    mapper.map_kernel_data(stack_start, stack, 8, true, false);
    debug!(
        "Kernel Stack Start: {:#x} End: {:#x}",
        stack_start, stack_end
    );

    info!("Allocating Kernel memory map...");
    let memory = st
        .boot_services()
        .allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1)
        .unwrap();
    let memory = unsafe {
        slice::from_raw_parts_mut(
            memory as *mut MemoryDescriptor,
            4096 / mem::size_of::<MemoryDescriptor>(),
        )
    };
    debug!(
        "Allocated space for {} MemoryDescriptors",
        4096 / mem::size_of::<MemoryDescriptor>()
    );

    let (tls_length, tls_address) = tls
        .map(|x| (x.len() as u64, Box::leak(x).as_ptr() as u64))
        .unwrap_or_default();
    let mut info = Box::leak(Box::new(BootInfo {
        revision: CURRENT_REVISION,
        kernel_position: KernelPosition {
            tls_address,
            tls_length,
            stack_start,
            stack_end,
        },
        memory_info: MemoryInfo {
            descriptors: memory.as_mut_ptr(),
            count: 0,
        },
        system_table: 0,
        sdt_address,
        initfs_address: 0,
        initfs_length: 0,
    }));
    debug!("BootInfo: {:#x}", info as *const BootInfo as u64);

    let buffer = allocate_memory_map(st.boot_services()).unwrap();
    info!("Exiting boot services. Good bye!");
    let (st, iter) = st.exit_boot_services(image, buffer).unwrap();

    info.memory_info.count = iter.len() as u64;

    for (i, desc) in iter.enumerate() {
        let ty = match desc.ty {
            MemoryType::RESERVED => bootloader::MemoryType::Reserved,
            MemoryType::LOADER_CODE => bootloader::MemoryType::Available,
            MemoryType::LOADER_DATA => bootloader::MemoryType::Reserved,
            MemoryType::BOOT_SERVICES_CODE => bootloader::MemoryType::Available,
            MemoryType::BOOT_SERVICES_DATA => bootloader::MemoryType::Available,
            MemoryType::RUNTIME_SERVICES_CODE => bootloader::MemoryType::Reserved,
            MemoryType::RUNTIME_SERVICES_DATA => bootloader::MemoryType::Reserved,
            MemoryType::CONVENTIONAL => bootloader::MemoryType::Available,
            MemoryType::UNUSABLE => bootloader::MemoryType::Unuseable,
            MemoryType::ACPI_RECLAIM => bootloader::MemoryType::Acpi,
            MemoryType::ACPI_NON_VOLATILE => bootloader::MemoryType::AcpiNonVolatile,
            MemoryType::MMIO => bootloader::MemoryType::Reserved,
            MemoryType::MMIO_PORT_SPACE => bootloader::MemoryType::Reserved,
            MemoryType::PAL_CODE => bootloader::MemoryType::Reserved,
            MemoryType::PERSISTENT_MEMORY => bootloader::MemoryType::Reserved,
            _ => bootloader::MemoryType::Reserved,
        };

        memory[i] = MemoryDescriptor {
            ty,
            start_address: desc.phys_start,
            end_address: desc.phys_start + desc.page_count * 4096,
        };
    }

    info.system_table = st.get_current_system_table_addr();

    mapper.load();
    unsafe {
        core::arch::asm!(
            "mov rsp, {}",
            "mov rbp, rsp",
            "mov rdi, {}",
            "jmp {}",
            in(reg) stack_end,
            in(reg) info,
            in(reg) entry,
            options(noreturn)
        )
    }
}
