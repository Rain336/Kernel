#!/bin/bash
set -e

#echo Cleaning up...
#cargo clean

echo Building Kernel...
cargo build -p platform-x86_64

#echo Building UEFI Bootloader...
#cargo build -p bootloader_uefi --target x86_64-unknown-uefi

echo Creating EFI System Partition...
[ ! -d image ] && mkdir -p iso_root/system
#cp target/x86_64-unknown-uefi/debug/bootloader_uefi.efi image/EFI/BOOT/BOOTX64.EFI
cp target/x86_64-unknown-none/debug/platform-x86_64 iso_root/system/kernel
cp crates/platform-x86_64/limine.cfg iso_root/limine.cfg
cp /opt/limine/share/limine/limine.sys iso_root/limine.sys
cp /opt/limine/share/limine/limine-cd.bin iso_root/limine-cd.bin
cp /opt/limine/share/limine/limine-cd-efi.bin iso_root/limine-cd-efi.bin
#echo 'EFI\BOOT\BOOTX64.EFI' > image/startup.nsh
#hdiutil create -fs fat32 -ov -format UDTO -srcfolder image esp.cdr
xorriso -as mkisofs -b limine-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot limine-cd-efi.bin -efi-boot-part --efi-boot-image --protective-msdos-label iso_root -o microdragon.iso
/opt/limine/bin/limine-deploy microdragon.iso
rm -rf iso_root
#trap 'rm esp.cdr; exit' INT

#qemu-system-x86_64 -cpu qemu64 \
#  -drive if=pflash,format=raw,readonly=on,file=/Users/rain/OVMF_CODE.cc.fd \
#  -drive if=pflash,format=raw,file=/Users/rain/OVMF_VARS.fd \
#  -drive file=./microdragon.iso,if=ide \
#  -net none \
#  -d int \
#  -serial stdio \
#  -no-reboot 2> output.log

qemu-system-x86_64 -M q35 -m 2G -cdrom microdragon.iso -boot d -serial stdio -no-reboot

rm microdragon.iso