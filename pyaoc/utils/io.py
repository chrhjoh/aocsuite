from dataclasses import dataclass
import logging
import os
from typing import Iterable, List,  Optional
from pyaoc.utils import messages
from pathlib import Path

logger = logging.getLogger()

FileContent = str
    
@dataclass
class File():
    name: str
    content: FileContent

    def save(self) -> None:
        open(self.name, 'w').write(self.content)
        logger.debug(messages.FILE_SAVED.format(file=self.name))
 
    @classmethod
    def load(cls, path: str):
        content = open(path, 'r').read().strip()
        return cls(path, content)

class AocDirectory(Path):
    def __init__(self, directory: str, year: int, day: int) -> None:
        directory = str(Path(directory) / str(year) / str(day))
        super().__init__(directory)
        self.directory = directory

    def path_in_directory(self, filename: str):
        return self.directory + f'/{filename}'

    def load_file(self, filename: str) -> File:
        path = self.path_in_directory(filename)
        if not os.path.exists(path):
            raise FileNotFoundError(f'It seems you have not created an answer file. Was looking for file at {path}')
        return File.load(path)
    
    def save_files_in_dir(self, files: Iterable[File], skip_if_exists: Optional[List[str]]=None):
        if not skip_if_exists:
            skip_if_exists= []

        for file in files:
            self.save_file_in_dir(file, skip_exists=file.name in skip_if_exists)
            
    def save_file_in_dir(self, file: File, skip_exists: bool) -> None:
        file.name = self.path_in_directory(file.name)

        if not skip_exists or not os.path.exists(file.name):
            file.save()
        else:
            logger.info(messages.FILE_NOT_SAVED_SKIPPED.format(file=file.name))


    
