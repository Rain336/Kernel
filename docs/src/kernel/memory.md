# The Kernel Memory Subsystem

The memory subsystem should handle handing out pages to userspace applications and kernel subsystems.
For this to works the Subsystem has three key points it must handle:

## Free Page Tracking

The subsystem needs to keep track of the available memory.
This is done using a B-Tree keeping track of the free ranges of physical memory.
When the B-Tree is running out stale memory has to be swaped to disk. (Not yet implemented. / How does the kernel interact with the FS service?)

## Page Table Updates

The kernel needs to update the processors page tables for changes to take effect.
Here we diffiranciate between Kernel and User-space page Tables.
The kernel has one set of Page Tables shared by all processors,
while each userspace process has it's own set of Page Tables.
This means that Kernel Pages have to be updated atomically,
while userspace processes, bound to one processor, can be updated by their owning processor.

## Kernel Allocation

The kernel needs to allocate memory, so the memory subsystem needs to supply an allocator for it.
The kernel allocator has four sizes (64, 128, 256, 512) of fixed-sized allocators for smaller sized allocations,
with bigger sizes being allocated page-wise.
Allocation is mostly lock-free, except in the case where a new page needs to be requested,
with freeing being completely lock-free for the fixed-sized allocators.
Page-wise allocation always requires locking, since it's interacting with the free page tracker.
