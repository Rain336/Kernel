from argparse import ArgumentParser, FileType, Namespace

def parse_arguments() -> Namespace:
    build_options = ArgumentParser(add_help=False)
    build_options.add_argument('-t', '--target', default='x86_64', choices=['x86_64', 'riscv', 'aarch64'],
                        help='Specifies the Architecture to build for. (default: %(default)s) (possible values: %(choices)s)', metavar='ARCH')
    build_options.add_argument('-b', '--bootloader', default='Limine', choices=['Limine', 'Rust'],
                        help='Specifies the bootloader to use. (default: %(default)s) (possible values: %(choices)s)', metavar='LOADER')
    build_options.add_argument('-r', '--release', action='store_true', help='Does a release build of the kernel.')

    parser = ArgumentParser(description='Helper program to build and develop the microdragon kernel')
    sub = parser.add_subparsers(title='Commands', required=True, dest='action')

    sub.add_parser('build', parents=[build_options],
                   help='Builds the kernel', description='Builds the kernel using cargo. The output will be an ELF file under the targets directory.')

    pack = sub.add_parser('pack', parents=[build_options],
                          help='Packages the kernel', description='Packs the kernel and bootloader into a hybrid iso image. Builds the kernel before packaging.')
    pack.add_argument('-o', '--output', default='microdragon.iso', type=FileType('wb'),
                      help='Path to the iso file to create. (default: %(default)s)')
    pack.add_argument('-f', '--force', action='store_true', help='Forces a new disk directory to be crated.')

    run = sub.add_parser('run', help='Runs the kernel', description='Runs the kernel in a virtual machine. Will use the supplied iso file to boot.')
    run.add_argument('-f', '--firmware', default='Bios', choices=['Bios', 'Uefi'],
                     help='Firmware to run in the virtual machine. (default: %(default)s) (possible values: %(choices)s)', metavar='FIRMWARE')
    
    return parser.parse_args()