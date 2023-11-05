# Microdragon Kernel

> **A microkernel written in Rust, trying to bridge the gap between embedded and general operating systems**

[![License](https://img.shields.io/github/license/Microdragon/Kernel?style=flat-square)](LICENSE)

This repo contains the source code of the microdragon kernel, it's built-in modules and bootloader interfaces.

**NOTE:** if you plan on contributing to microdragon, feel free to read [CONTRIBUTING.md](CONTRIBUTING.md) too.

## Building from source

The microdragon kernel is build using [Just](https://github.com/casey/just), a simple command runner that can be installed using cargo:
> cargo install just

With just installed, it's as simple as running `just build` to build the kernel binary.

By default just will do a debug build for x86_64 systems and the limine bootloader.
This can be caged by passing `<key>=<value>` pairs to just.
|Key|Default|Description|
|-|-|-|
|bootloader|limine|Bootloader to support|
|target|x86_64-unknown-none|Rust target|
|release|false|Build as Release?|

## Packaging the kernel

The kernel binary alone doesn't get you far, so the next step is to package up the kernel into a `.iso` image.
Here just comes to help again by running `just pack`, just will build and package your project together with the selected bootloader.
The resulting `.iso` can be booted by both a legacy bios system as well as a UEFI system.
**NOTE:** To create the `.iso` a tool called `xorriso` might be needed.

## Running the kernel in QEMU

Finally if you just want to test microdragon out or are tinkering on it, just also provides a `just run_bios` and a `just run_uefi` command.
It will build, package and then run QEMU for the given target.
