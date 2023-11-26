from pycdlib import PyCdlib
from .bootloader import Bootloader, BootloaderRegistry
import os

class LimineBootloader(Bootloader):
    def __init__(self) -> None:
        self.project = f'microdragon-limine'
    
    def copy_files(self):
        target = os.path.join('disk', 'limine')
        self.ensure_directory(target)

        self.test_or_copy(os.path.join(target, 'limine-uefi-cd.bin'), os.path.join('deps', 'limine', 'limine-uefi-cd.bin'))
        self.test_or_copy(os.path.join(target, 'limine-bios-cd.bin'), os.path.join('deps', 'limine', 'limine-bios-cd.bin'))
        self.test_or_copy(os.path.join(target, 'limine-bios.sys'), os.path.join('deps', 'limine', 'limine-bios.sys'))
        self.test_or_copy(os.path.join(target, 'limine.cfg'), os.path.join('bootloader', 'limine', 'limine.cfg'))

        efi = os.path.join('disk', 'EFI', 'BOOT')
        self.ensure_directory(efi)

        self.test_or_copy(os.path.join(efi, 'BOOTAA64.EFI'), os.path.join('deps', 'limine', 'BOOTAA64.EFI'))
        self.test_or_copy(os.path.join(efi, 'BOOTRISCV64.EFI'), os.path.join('deps', 'limine', 'BOOTRISCV64.EFI'))
        self.test_or_copy(os.path.join(efi, 'BOOTX64.EFI'), os.path.join('deps', 'limine', 'BOOTX64.EFI'))
    
    def make_bootable(self, iso: PyCdlib):
        print('Creating El Torito boot entry for bios...')
        record = iso.get_record(rr_path='/limine/limine-bios-cd.bin')
        iso.add_eltorito('/LIMINE/' + record.file_ident.decode('utf-8'), media_name='noemul', boot_load_size=4, boot_info_table=True)
        
        print('Creating El Torito boot entry for UEFI...')
        record = iso.get_record(rr_path='/limine/limine-uefi-cd.bin')
        iso.add_eltorito('/LIMINE/' + record.file_ident.decode('utf-8'), media_name='noemul', boot_load_size=4, boot_info_table=True, efi=True)

BootloaderRegistry().register('Limine', LimineBootloader())
