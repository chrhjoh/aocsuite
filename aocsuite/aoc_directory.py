import logging
import os
import shutil
from collections.abc import Mapping
from pathlib import Path
from typing import override

from aocsuite.utils import messages
from aocsuite.utils.filenames import EXAMPLE_FILE, INPUT_FILE, PUZZLE_FILE

logger = logging.getLogger(__file__)


class AocDataDirectory:
    def __init__(self, base_dir: str, year: int, day: int, force: bool) -> None:
        self.base_dir: Path = Path(base_dir).resolve()
        self.force: bool = force
        self.year: int = year
        self.day: int = day

    @property
    def directory(self) -> Path:
        return self.base_dir / str(self.year) / str(self.day)

    def save_files(self, files: Mapping[str, str], **kwargs: bool):
        os.makedirs(self.directory, exist_ok=True)
        for name, content in files.items():
            path = self.directory / name
            logger.debug(
                messages.DEBUG_PARSED_AOC_FILES.format(
                    input_type=path, sample=content[:200]
                )
            )
            self.save_file(path, content, **kwargs)

    def copy_to_workdir(self, files: list[Path]) -> None:
        for from_file in files:
            to_file = self.directory / from_file.name
            if verify_file_save(Path(to_file)) or self.force:
                shutil.copy(from_file, to_file)

    def copy_to_basedir(self, files: list[Path]) -> None:
        for from_file in files:
            to_file = self.base_dir / from_file.name
            if verify_file_save(Path(to_file)) or self.force:
                shutil.copy(from_file, to_file)

    def save_file(self, name: Path, content: str, force: bool = False) -> None:
        if self.force or force or verify_file_save(name):
            _ = open(name, "w").write(content)

    def exists(self):
        return self.directory.exists()

    def is_initialized(self):
        return (
            (self.directory / INPUT_FILE).exists()
            and (self.directory / EXAMPLE_FILE).exists()
            and (self.directory / PUZZLE_FILE).exists()
        )

    @override
    def __str__(self) -> str:
        return str(self.directory)

    def __truediv__(self, path_part: str):
        return self.directory / path_part


def verify_file_save(path: Path) -> bool:
    if not path.exists():
        return True

    response = ""
    while response.lower() not in ["y", "n"]:
        response = input(
            f"File {path} already exists. Do you want to overwrite it? [y/n]"
        )
    return response.lower() == "y"
