from pyaoc.utils.io import File
from dataclasses import dataclass
import logging
from typing import Iterable

logger = logging.getLogger(__file__)

@dataclass
class Template():
    name : str
    filetype : str
    utils: str
    exercise: str = ''

    def __post_init__(self):
        TEMPLATES[self.name] = self

    def construct_template_files(self, base_path: str) -> Iterable[File]:
        exercise1 = File(name=f'{base_path}/exercise1{self.filetype}',
                         content = self.exercise) 
        exercise2 = File(name=f'{base_path}/exercise2{self.filetype}',
                         content = self.exercise)
        utils = File(name=f'{base_path}/utils{self.filetype}',
                         content = self.exercise)
        return [exercise1, exercise2, utils]

TEMPLATES: dict[str, Template] ={}

def create_language_template(directory: str, language: str) -> None:
    if language not in TEMPLATES:
        raise NotImplementedError(f'Template for language {language} does not exist. Supported languages are {TEMPLATES.keys()}')

    template = TEMPLATES[language]
    files_to_save = template.construct_template_files(directory)
    for file in files_to_save:
        file.save()







