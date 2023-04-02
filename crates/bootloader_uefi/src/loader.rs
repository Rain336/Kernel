use crate::mapper::{MemoryMapper, KERNEL_CODE_START};
use alloc::boxed::Box;
use bootloader::KernelEntryPointFn;
use core::mem;
use core::slice;
use elf::file::Architecture;
use elf::gabi::{PF_W, PF_X, PT_LOAD, PT_TLS, SHT_RELA};
use elf::{File, ParseError};
use log::debug;
use uefi::prelude::BootServices;
use uefi::table::boot::{AllocateType, MemoryType};

fn native_arch() -> Architecture {
    #[cfg(target_arch = "x86_64")]
    return Architecture(elf::gabi::EM_X86_64);
    #[cfg(target_arch = "aarch64")]
    return Architecture(elf::gabi::EM_AARCH64);
    #[cfg(target_arch = "riscv")]
    return Architecture(elf::gabi::EM_RISCV);
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv")))]
    compile_error!("Unsupported UEFI target")
}

pub fn load_kernel(
    kernel: &[u8],
    mapper: &mut MemoryMapper,
    boot: &BootServices,
) -> Result<(KernelEntryPointFn, Option<Box<[u8]>>), ParseError> {
    let mut file = File::open_stream(kernel)?;

    assert_eq!(
        file.ehdr.arch,
        native_arch(),
        "Kernel and Machine arcitectures don't match"
    );

    assert_ne!(file.ehdr.e_entry, 0, "Kernel doesn't have an entry point");

    let mut tls = None;
    for segment in file.segments()? {
        if segment.p_type == PT_TLS {
            tls = Some(
                kernel[segment.p_offset as usize..(segment.p_offset + segment.p_filesz) as usize]
                    .into(),
            );
            continue;
        }

        if segment.p_type != PT_LOAD {
            continue;
        }

        assert_eq!(
            segment.p_align, 4096,
            "Kernel Segments should be 4096 bytes aligned"
        );

        debug!(
            "Kernel Segment at {:#x} with length {}",
            segment.p_vaddr + KERNEL_CODE_START,
            segment.p_memsz,
        );

        let writeable = (segment.p_flags.0 & PF_W) == PF_W;
        let executable = (segment.p_flags.0 & PF_X) == PF_X;
        let offset = segment.p_vaddr & 0o7777;
        let pages = (segment.p_memsz + offset + 4095) >> 12;

        let ptr = boot
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                pages as usize,
            )
            .unwrap();
        let buffer = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, pages as usize * 4096) };
        buffer.fill(0);
        buffer[offset as usize..(segment.p_filesz + offset) as usize].copy_from_slice(
            &kernel[segment.p_offset as usize..(segment.p_offset + segment.p_filesz) as usize],
        );

        mapper.map_kernel_code(
            segment.p_vaddr + KERNEL_CODE_START,
            ptr,
            pages,
            writeable,
            executable,
        );
    }

    //let mut rel = None;
    let mut rela = None;
    for header in file.section_headers()? {
        //if header.sh_type == SHT_REL {
        //    rel.replace(header);
        //} else
        if header.sh_type == SHT_RELA {
            rela.replace(header);
        }
    }

    if let Some(rela) = rela {
        debug!("Found Rela section...");
        for entry in file.section_data_as_relas(&rela)? {
            assert_eq!(
                entry.r_sym, 0,
                "Relocations with Symbol Table lookup aren't supported"
            );

            // TODO: Type is specific to x86_64
            assert_eq!(
                entry.r_type, 8,
                "Only Relocation type 8 is supported for x86_64"
            );

            let address = KERNEL_CODE_START.checked_add(entry.r_offset).unwrap();
            let value = KERNEL_CODE_START.checked_add(entry.r_addend as u64).unwrap();

            let physical = mapper.translate(address);
            assert_ne!(physical, 0, "Relocation outside of loaded segment");
            assert_eq!(
                physical % mem::size_of::<u64>() as u64,
                0,
                "Relocation destination is unailinged"
            );

            //debug!("Patching {:#x}->{:#x} to {}", physical, address, value);
            unsafe { (physical as *mut u64).write(value) };
        }
    }

    Ok((
        unsafe { mem::transmute(file.ehdr.e_entry + KERNEL_CODE_START) },
        tls,
    ))
}
