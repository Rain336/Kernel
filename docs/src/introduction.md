## Microkernel

Microdragon is a microkernel, a type of kernel that tries to off-load most of it's work into userspace.
Microdragon only does scheduling, thread and process management as well as memory management in kernel.
All other operations like device drivers, filesystem interaction, etc. should be done by userspace services.