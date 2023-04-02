#!/bin/bash
set -e

#echo Cleaning up...
#cargo clean

echo Building Kernel...
cargo build -p platform-x86_64

echo Building UEFI Bootloader...
cargo build -p bootloader_uefi --target x86_64-unknown-uefi

echo Creating EFI System Partition...
[ ! -d image ] && mkdir -p image/EFI/BOOT && mkdir -p image/system
cp target/x86_64-unknown-uefi/debug/bootloader_uefi.efi image/EFI/BOOT/BOOTX64.EFI
cp target/x86_64-unknown-none/debug/platform-x86_64 image/system/kernel
echo 'EFI\BOOT\BOOTX64.EFI' > image/startup.nsh
hdiutil create -fs fat32 -ov -format UDTO -srcfolder image esp.cdr
rm -rf image
#trap 'rm esp.cdr; exit' INT

clear
qemu-system-x86_64 -cpu qemu64 \
  -drive if=pflash,format=raw,readonly=on,file=/Users/rain/OVMF_CODE.cc.fd \
  -drive if=pflash,format=raw,file=/Users/rain/OVMF_VARS.fd \
  -drive file=./esp.cdr,if=ide \
  -net none \
  -d int \
  -serial stdio \
  -no-reboot 2> output.log

rm esp.cdr