from abc import ABC, abstractmethod
from typing import Optional, Self
from pycdlib import PyCdlib
import os
import shutil

class Bootloader(ABC):
    project: str

    @abstractmethod
    def copy_files(self):
        pass

    @abstractmethod
    def make_bootable(self, iso: PyCdlib):
        pass

    def ensure_directory(self, path: str):
        if not os.path.exists(path):
            os.makedirs(path)
    
    def test_or_copy(self, target: str, copy: str):
        if not os.path.exists(target) or os.path.getmtime(copy) > os.path.getmtime(target):
            print(f'Copying file from {copy} to {target}...')
            shutil.copyfile(copy, target)

class BootloaderRegistry:
    INSTANCE: Optional[Self] = None
    bootloaders: dict[str, Bootloader]

    def __new__(cls) -> Self:
        if cls.INSTANCE is None:
            cls.INSTANCE = super(BootloaderRegistry, cls).__new__(cls)
            cls.INSTANCE.bootloaders = {}
        return cls.INSTANCE
    
    def register(self, name: str, loader: Bootloader):
        if name in self.bootloaders:
            raise RuntimeError(f'Bootloader {name} is already registered.')
        
        if not isinstance(loader, Bootloader):
            raise TypeError('Given bootloader does not derive from Bootloader class')
        
        self.bootloaders[name] = loader
    
    def get_bootloader(self, name: str) -> Bootloader:
        if name not in self.bootloaders:
            raise RuntimeError(f'No bootloader registered with name {name}')
        
        return self.bootloaders[name]
