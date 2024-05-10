import logging
from time import time
from typing import Callable, Optional, Type

from pyaoc.utils import messages
from pyaoc.utils.enums import LanguageName

logger = logging.getLogger(__file__)


def log_executor(executor: Callable):
    def wrapper(*args, **kwargs):
        logger.info(messages.EXECUTOR_RUN_CMD.format(executor=executor.__name__))
        start_time = time()
        executor(*args, **kwargs)
        run_time = time() - start_time
        logger.info(messages.EXECUTOR_FINISHED_CMD.format(seconds=run_time))

    return wrapper


class LanguageAdapter:

    def __init__(self, directory: str) -> None:
        self.directory = directory

    def initialize(self):
        raise NotImplementedError("Should be implemented for each Language")

    @log_executor
    def execute(self, exercise: int, data_path: str, answer_path: Optional[str]) -> None:
        raise NotImplementedError("Should be implemented for each Language")


LANGUAGES = {}


def register_language(language: LanguageName):
    def _wrapper(cls: Type[LanguageAdapter]):
        LANGUAGES[language] = cls

    return _wrapper


def get_language(language: LanguageName, **kwargs) -> LanguageAdapter:
    if language is None:
        logger.warning(messages.DEFAULT_LANGUAGE_MESSAGE)
        return LANGUAGES[LanguageName.PYTHON](**kwargs)

    if language not in LANGUAGES:
        raise NotImplementedError(
            messages.LANGUAGE_NOT_FOUND_MESSAGE.format(
                language=language, supported=LANGUAGES.keys()
            )
        )
    return LANGUAGES[language](**kwargs)
