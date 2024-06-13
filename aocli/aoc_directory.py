import logging
import os
import shutil
from pathlib import Path
from typing import List, Mapping

from aocli.utils import filenames, messages
from aocli.utils.enums import InputType

logger = logging.getLogger(__file__)


class AocDirectory:
    def __init__(self, base_dir: str, year: int, day: int, force: bool) -> None:
        self.base_dir = Path(base_dir).resolve()
        self.force = force
        self.year = year
        self.day = day

    @property
    def directory(self) -> Path:
        return self.base_dir / str(self.year) / str(self.day)

    def data_path(self, input_type: InputType) -> Path:
        filname = (
            filenames.INPUT_FILE
            if input_type == InputType.INPUT
            else filenames.EXAMPLE_FILE
        )

        return self.directory / filname

    def save_files(self, files: Mapping[str, str], **kwargs):
        for name, content in files.items():
            path = self.directory / name
            logger.debug(
                messages.DEBUG_PARSED_AOC_FILES.format(
                    input_type=path, sample=content[:200]
                )
            )
            self.save_file(path, content, **kwargs)

    def copy_files(self, files: List[Path]) -> None:
        for from_file in files:
            to_file = self.directory / from_file.name
            if self.verify_file_save(Path(to_file)) or self.force:
                shutil.copy(from_file, to_file)

    def save_file(self, name: Path, content: str, force: bool = False) -> None:
        if self.force or force or self.verify_file_save(name):
            open(name, "w").write(content)

    def exists(self):
        return self.directory.exists()

    def initialize(self):
        os.makedirs(self.directory, exist_ok=True)

    def __str__(self) -> str:
        return str(self.directory)

    def __truediv__(self, path_part: str):
        return self.directory / path_part

    def verify_file_save(self, path: Path) -> bool:
        if not path.exists():
            return True

        response = ""
        while response.lower() not in ["y", "n"]:
            response = input(
                f"File {path} already exists. Do you want to overwrite it? [y/n]"
            )
        return response.lower() == "y"
