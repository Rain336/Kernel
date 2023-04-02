# The Kernel Logging Subsystem

Currently, the kernel logging subsystem is kept very simple by just writing out to the first serial uart port of the machine.
The logging subsystem integrates with the `log` crate, so it's logging macros can be used for simple logging.

## Future Plans

### Framebuffer Logging with buffer from `Limine`

The Kernel will be switching to the `Limine Boot Protocol` soon, wich allows us to request a framebuffer from the bootloader.
This will allow us to also print to the screen in addition to just uart serial.
But this will also require a point at wich the framebuffer must be given up to the userspace.
Maybe this can be combined with the switch to a logging system?

### Switch to the `tracing` crate

The tracing crate allows collecting spans of time in addition to simple logging events.
It supports the same macros as the `log` crate, so can be an easy upgrade from it.
The early kernel has no time tracking capabilities, so early span collection is limited.

### Userspace logging system

A userspace service should take over log collection in the future.
This allows the serial port used for logging to be freed up for userspace usage.
If the logging service is a central log collecter, like Windows EventLog or just for kernel logging like Linux Syslog
is left up to the service implementation and the kernel should not assume either.
