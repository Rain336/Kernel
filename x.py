#!/usr/bin/env python3
from __future__ import print_function

if __name__ == '__main__':
    import sys

    major = sys.version_info.major
    minor = sys.version_info.minor
    
    if major != 3 or minor < 4:
        print('Python 3.4 or higher is required to run x.py')
        exit(1)

    import scripts
    scripts.main()
