from argparse import ArgumentParser, Namespace
from .bootloader import Bootloader, BootloaderRegistry
from .task import Task, register_task

RUST_ARCH_MAPPING = {
    'x86_64': 'x86_64-unknown-none',
    'aarch64': 'arch64-unknown-none-softfloat',
    'riscv': 'riscv64imac-unknown-none-elf',
}

def to_rust_arch(arch) -> str:
    assert arch in RUST_ARCH_MAPPING, f'Architecture mapping not defined for {arch}'
    return RUST_ARCH_MAPPING[arch]

@register_task('build')
class BuildTask(Task):
    target: str
    bootloader: Bootloader
    release: bool

    def __init__(self):
        self.target = 'x86_64-unknown-none'
        self.bootloader = BootloaderRegistry().get_bootloader('Limine')
        self.release = False

    @classmethod
    def configure_argparse(cls, parser: ArgumentParser):
        parser.description = 'Builds the kernel using cargo. The output will be an ELF file under the targets directory.'
        
        parser.add_argument('-t', '--target', default='x86_64', choices=['x86_64', 'riscv', 'aarch64'],
                        help='Specifies the Architecture to build for. (default: %(default)s) (possible values: %(choices)s)', metavar='ARCH')
        parser.add_argument('-b', '--bootloader', default='Limine', choices=['Limine', 'Rust'],
                        help='Specifies the bootloader to use. (default: %(default)s) (possible values: %(choices)s)', metavar='LOADER')
        parser.add_argument('-r', '--release', action='store_true', help='Does a release build of the kernel.')
    
    def extract_arguments(self, args: Namespace):
        self.target = to_rust_arch(args.target)
        self.bootloader = BootloaderRegistry().get_bootloader(args.bootloader)
        self.release = args.release
    
    def run(self):
        cargo = ['cargo', 'build', '--target', self.target, '-p', self.bootloader.project]

        if self.release:
            cargo.append('-r')
        
        self.run_command(cargo)