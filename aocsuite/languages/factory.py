import logging
import os
import shutil
import subprocess
import sys
from pathlib import Path
from time import time
from typing import Callable, List, Optional, Type

from aocsuite.aoc_directory import verify_file_save
from aocsuite.utils import messages
from aocsuite.utils.enums import LanguageName

LANGUAGES = {}

logger = logging.getLogger(__file__)


def log_run(runner: Callable):
    def wrapper(*args, **kwargs):
        logger.info(messages.RUN_START_CMD)
        start_time = time()
        runner(*args, **kwargs)
        run_time = time() - start_time
        logger.info(messages.RUN_FINISHED_CMD.format(seconds=run_time))

    return wrapper


class LanguageAdapter:
    def __init__(
        self, name: LanguageName, base_dir: str, year: int, day: int, force=bool
    ) -> None:
        self.template_directory = Path(__file__).parent.parent / "__assets__" / name
        self.language_base_dir = Path(base_dir) / name
        self.force_files = force

    def apply_base_template(self) -> None:
        raise NotImplementedError("Should be implemented for each Language")

    def apply_exercise_template(self) -> None:
        raise NotImplementedError("Should be implemented for each Language")

    def command(self, exercise: int, data_path: str) -> List[str]:
        raise NotImplementedError("Should be implemented for each Language")

    def get_exercise_path(self) -> str:
        raise NotImplementedError("Should be implemented for each Language")

    def is_initialized(self) -> bool:
        raise NotImplementedError("Should be implemented for each Language")

    def copy_to_directory(
        self,
        from_file: Path,
        to_file: Path,
    ) -> None:
        os.makedirs(to_file.parent, exist_ok=True)
        if verify_file_save(Path(to_file)) or self.force_files:
            shutil.copy(from_file, to_file)

    @log_run
    def __call__(self, *args, **kwargs) -> None:
        """
        Takes args and kwargs and passes it to the command. And then runs the command generated by the command method.
        """
        try:
            subprocess.run(self.command(*args, **kwargs), check=True)
        except subprocess.CalledProcessError as err:
            logger.error(
                f"Error was encountered while running exericse. See traceback above. Error Encountered:\n{err}"
            )
            sys.exit(1)


def register_language(language: LanguageName):
    def _wrapper(cls: Type[LanguageAdapter]):
        LANGUAGES[language] = cls

    return _wrapper


def get_language(language: Optional[LanguageName], **kwargs) -> LanguageAdapter:
    if language is None:
        language = LanguageName.PYTHON
        logger.warning(messages.DEFAULT_LANGUAGE_MESSAGE)

    if language not in LANGUAGES:
        raise NotImplementedError(
            messages.LANGUAGE_NOT_FOUND_MESSAGE.format(
                language=language, supported=LANGUAGES.keys()
            )
        )
    return LANGUAGES[language](name=language, **kwargs)
