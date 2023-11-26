from argparse import ArgumentParser, Namespace, FileType
from typing import Any
from .build import BuildTask
from .task import register_task
import os
import shutil
import pycdlib

VALID_ISO_CHARS = list(chr(i) for i in range(0x41, 0x5B)) + list(chr(i) for i in range(0x30, 0x3A)) + ['_']
ISO_NUMBERING = {}

def make_iso_compliant(file: str):
    (file, ext) = os.path.splitext(file)
    file = file.upper()
    ext = ext.upper()

    name = []
    for c in file:
        if c in VALID_ISO_CHARS:
            name.append(c)
        else:
            name.append('_')
    name = ''.join(name)

    if len(name) > 8 or name != file:
        name = name[:6]
        if name in ISO_NUMBERING:
            number = ISO_NUMBERING[name]
            ISO_NUMBERING[name] += 1
        else:
            number = 1
            ISO_NUMBERING[name] = 2
        name += f'{number:02d}'
    
    if len(ext) > 4 or any(c for c in ext[1:] if c not in VALID_ISO_CHARS):
        ext = '.'
    
    return name + ext

@register_task('pack')
class PackagingTask(BuildTask):
    output: Any
    force: bool

    def __init__(self) -> None:
        super().__init__()
        self.output = None
        self.force = False

    @classmethod
    def configure_argparse(cls, parser: ArgumentParser):
        super(PackagingTask, cls).configure_argparse(parser)
        parser.description = 'Packages the kernel and bootloader into a hybrid iso image. Builds the kernel before packaging.'

        parser.add_argument('-o', '--output', default='microdragon.iso', type=FileType('wb'),
                      help='Path to the iso file to create. (default: %(default)s)')
        parser.add_argument('-f', '--force', action='store_true', help='Forces a new disk directory to be crated.')
    
    def extract_arguments(self, args: Namespace):
        super().extract_arguments(args)
        self.output = args.output
        self.force = args.force
    
    def run(self):
        super().run()
        self._assemble_disk()
        self._package_disk_dir()
    
    def _package_disk_dir(self):
        iso = pycdlib.PyCdlib()
        iso.new(rock_ridge='1.09')

        for (root, dirs, files) in os.walk('disk'):
            if os.path.sep != '/':
                disk_root = root.replace(os.path.sep, '/')
            else:
                disk_root = root

            disk_root = disk_root[4:]

            iso_disk_root = '/'.join([s.upper() for s in disk_root.split('/')])

            for dir in dirs:
                print(f'Creating iso directory {iso_disk_root}/{dir.upper()} rr: {dir}')
                iso.add_directory(f'{iso_disk_root}/{dir.upper()}', rr_name=f'{dir}')

            for file in files:
                iso_file = make_iso_compliant(file)
                print(f'Creating iso file {iso_disk_root}/{iso_file};1 rr: {file}')
                iso.add_file(os.path.join(root, file), f'{iso_disk_root}/{iso_file};1', rr_name=f'{file}')
    
        self.bootloader.make_bootable(iso)

        iso.write_fp(self.output)
        iso.close()
    
    def _assemble_disk(self):
        if self.force and os.path.exists('disk'):
            print('Cleaning existing disk folder...')
            shutil.rmtree('disk')

        if not os.path.exists('disk'):
            os.mkdir('disk')
        
        self._copy_kernel()
        self.bootloader.copy_files()
    
    def _copy_kernel(self):
        system = os.path.join('disk', 'system')
        if not os.path.exists(system):
            os.mkdir(system)

        if self.release:
            mode = 'release'
        else:
            mode = 'debug'

        print('Copying kernel binary...')
        shutil.copyfile(os.path.join('target', self.target, mode, self.bootloader.project), os.path.join(system, 'kernel'))
