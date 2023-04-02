# Bootloader Interface

## Abstract
This paper defines the way a bootloader can load and run the microdragon kernel.
Everything not explicitly mentioned here is up to the implementation of the bootloader and should generally not be relied upon by the kernel.
*Text in italic are draft notes left by editors and should not be kept in the final version.*

## Changelog
|Revision|Notes|
|-|-|
|Revision 1 (Draft)|Initial Release|

## Finding the kernel and initfs
How and where the bootloader finds the kernel file is mostly left up to the implementation itself,
but some guidelines are given. In secenarios where the bootloader has access to a filesystem (UEFI f.e.),
the kernel and initfs should be located on it at `/system/kernel` and `/system/initfs` respectivly.
In other scenarios we suggest that the kernel should be inside the initfs and loaded that way.
We cannot cover all possible scenarios, which is why we ultimatly leave most this up to the bootloader,
the kernel doesn't care where it comes from, just that it exists and has access to an initfs.

*Maybe we should offer a build of the kernel with the init process copied into, similar to how FreeRTOS allowes to compile the process into the kernel*

## Loading the kernel
The kernel is an ELF executeable that is either a staticly or dynamicly positioned.
It is up to the bootloader to correctly detect and load the kernel at it's expected position.
In case of a dynamicly positioned kernel, the kernel should be loaded into high memory (0xFF8000000000 - 0xFFFFFFFFFFFF).
The exact location is left up to the bootloader to implement security featuers like ASLR.

## Kernel stack
The kernel needs a stack to be called.
This stack is passed through the [`KernelPosition`](#kernelposition-strucure) struct.
The stack should be at least 16KiB in size.

## Switching to the kernel
The kernel executeable exposes an entry point that should be call to switch over to the kernel.
The entry point function uses the System V ABI and only takes a pointer to the [`BootInfo`](#bootinfo-strucure) struct defined later in this document.
The entry point function never returns.

## Virtual memory
Before switching to the kernel...
- ...the whole physical memory should be identity mapped to virtual memory.
- ...the kernel should be mapped into high memory (0xFF8000000000 - 0xFFFFFFFFFFFF).

*The difinition of high memory currently assumes a 64-bit system. Maybe 32-bit supported should be evaluated in the future*

## BootInfo Strucure
The `BootInfo` struct is the main way of how information is passed from the bootloader to the kernel.
This section will define multiple structs in a C like language and explain their fields afterwards.
ABI whise all structs should be using the standard C ABI.

```C
struct BootInfo {
    uint8_t revision,
    KernelPosition kernel_position,
    MemoryInfo memory_info,
    uint64_t system_table,
    uint64_t sdt_address,
    uint64_t initfs_address,
    uint64_t initfs_length
}
```

- `revision` The version of this document as implememnted by the bootloader. This is to catch version mismatches between the bootloader and the kernel.
- `kernel_position` Defined in [KernelPosition Strucure](#kernelposition-strucure).
- `memory_info` Defined in [MemoryInfo Structure](#memoryinfo-structure).
- `system_table` The physical address of the UEFI System Table, if the kernel was booted though UEFI, otherwise `0`.
- `sdt_address` The physical address of the ACPI Root/eXtended System Descriptor Table or `0` if unavilable.
- `initfs_address` The physical address of of where the bootloader loaded the initfs. If no initfs was supplied it should be `0`.
- `initfs_length` The length of the loaded initfs or `0` if no initfs was supplied.


## KernelPosition Strucure
The kernel position struct tells the kernel where it's stack and tls segment is located in physical memory.

```C
struct KernelPosition {
    uint64_t tls_address,
    uint64_t tls_length,
    uint64_t stack_start,
    uint64_t stack_end
}
```

- `tls_address` The physical addresses of the start of the kernel's tls segment. The kernel should only have one tls segment.
- `tls_length` The length in bytes of the kernel's tls segment.
- `stack_start` The address of the start of the kernel's stack.
- `stack_end` The address of the end of the kernel's stack.

## MemoryInfo Structure
The memory info struct tells the kernel about what regions of memory are freely useable and which ones are in use and by what.
The main `MemoryInfo` struct is just an array of `MemoryDescriptor`s.
A `MemoryDescriptor` defins a section of memory and says if it's available or what it is used by.

```C
struct MemoryInfo {
    uint64_t count,
    MemoryDescriptor* descriptors.
}
```
- `count` The length of the `descriptors` array, in number of `MemoryDescriptor` structs.
- `descriptors` An array of `MemoryDescriptor`s, describing the avilable memory.

```C
struct MemoryDescriptor {
    uint64_t type,
    uint64_t start_address,
    uint64_t end_address
}
```
- `type` Type is an enum that describes the availablility of this section.
- `start_address` The physical starting address of this section.
- `end_address` The physical ending address of this section.

```C
enum MemoryType {
    RESERVED = 0,
    UNUSEABLE = 1,
    ACPI = 2,
    ACPI_NON_VOLATILE = 3,
    AVAILABLE = 4
}
```
- `RESERVED` Reserved memory is used in some form. Either through MMIO or the kernel.
- `UNUSEABLE` Unuseable is memory where errors have been detected and consiquently shouldn't be written to.
- `ACPI` ACPI memory contins ACPI tables that can be recalimed after the tables have been read out.
- `ACPI_NON_VOLATILE` ACPI non volatile memory has to be saved during power state changes.
- `AVAILABLE` Available memory can be freely used by the kernel.