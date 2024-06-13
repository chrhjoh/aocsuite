import logging
import subprocess
import sys
from pathlib import Path
from time import time
from typing import Callable, List, Optional, Type

from aocli.aoc_directory import AocDirectory
from aocli.utils import messages
from aocli.utils.enums import LanguageName

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
    def __init__(self, name: LanguageName, working_directory: AocDirectory) -> None:
        self.template_directory = Path(__file__).parent / "templates" / name
        self.working_directory = working_directory

    def fetch(self):
        raise NotImplementedError("Should be implemented for each Language")

    def command(self, exercise: int, data_path: str) -> List[str]:
        raise NotImplementedError("Should be implemented for each Language")

    def get_exercise_name(self):
        raise NotImplementedError("Should be implemented for each Language")

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


def get_language(
    language: Optional[LanguageName], working_directory: AocDirectory, **kwargs
) -> LanguageAdapter:
    if language is None:
        language = auto_detect_language(working_directory)

    if language is None:
        language = LanguageName.PYTHON
        logger.warning(messages.DEFAULT_LANGUAGE_MESSAGE)

    if language not in LANGUAGES:
        raise NotImplementedError(
            messages.LANGUAGE_NOT_FOUND_MESSAGE.format(
                language=language, supported=LANGUAGES.keys()
            )
        )
    return LANGUAGES[language](
        name=language, working_directory=working_directory, **kwargs
    )


def auto_detect_language(working_directory: AocDirectory) -> Optional[LanguageName]:
    for file in working_directory.directory.glob("*"):
        if file.suffix == ".rs":
            return LanguageName.RUST
        elif file.suffix == ".py":
            return LanguageName.PYTHON
    return None
