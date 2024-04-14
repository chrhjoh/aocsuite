from dataclasses import dataclass
import logging
from pathlib import Path

logger = logging.getLogger()

@dataclass
class File():
    name: str
    content: str

    def save(self) -> None:
        open(self.name, 'w').write(self.content)
        logger.debug(f'Saved file: {self.name}')


def load_answer(directory: str, exercise: int) -> int:
    answer_file = Path(directory) / f'answer{exercise}.txt'
    answer = open(answer_file, 'r').read().strip()
    return int(answer)
