from argparse import ArgumentParser, Namespace
from abc import ABC, abstractmethod
from typing import Self, Optional, Dict
import subprocess

class Task(ABC):
    @classmethod
    @abstractmethod
    def configure_argparse(cls, parser: ArgumentParser):
        pass

    @abstractmethod
    def extract_arguments(self, args: Namespace):
        pass

    @abstractmethod
    def run(self):
        pass

    def run_command(self, cmd: list[str]):
        print('Running command: ' + ' '.join(cmd))
        subprocess.run(cmd)

class TaskRegistry:
    INSTANCE: Optional[Self] = None
    tasks: Dict[str, type[Task]]

    def __new__(cls) -> Self:
        if cls.INSTANCE is None:
            cls.INSTANCE = super(TaskRegistry, cls).__new__(cls)
            cls.INSTANCE.tasks = {}
        return cls.INSTANCE
    
    def register(self, name: str, task: type[Task]):
        if name in self.tasks:
            raise RuntimeError(f'Task {name} is already registered.')
        
        if not issubclass(task, Task):
            raise TypeError('Given task does not derive from Task class')
        
        self.tasks[name] = task
    
    def build_argparse(self) -> ArgumentParser:
        parser = ArgumentParser(description='Helper program to build and develop the microdragon kernel')
        sub = parser.add_subparsers(title='Subcommands', required=True, dest='action')

        for (name, task) in self.tasks.items():
            child = sub.add_parser(name)
            task.configure_argparse(child)
        
        return parser
    
    def create_task(self, name: str) -> Task:
        if name not in self.tasks:
            raise RuntimeError(f'No task registered with name {name}')
        
        return self.tasks[name]()

def register_task(name: str):
    def decorator(ty: type[Task]):
        TaskRegistry().register(name, ty)
        return ty
    return decorator