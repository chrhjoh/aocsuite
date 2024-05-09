import logging
from dataclasses import dataclass

from pyaoc.utils import messages

logger = logging.getLogger()

FileContent = str


@dataclass
class File:
    name: str
    content: FileContent

    def save(self) -> None:
        open(self.name, "w").write(self.content)
        logger.debug(messages.FILE_SAVED.format(file=self.name))

    @classmethod
    def load(cls, path: str):
        content = open(path, "r").read().strip()
        return cls(path, content)
