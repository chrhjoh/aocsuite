from pyaoc.utils import messages
from time import time
import logging
from typing import Callable, Optional

logger = logging.getLogger(__file__)

CreateUtilsContent = Callable[[], str]
CreateExerciseContent = Callable[[], str]
Executor = Callable[[str, str, Optional[str]], int]
class Language():
    
    def __init__(self, name: str, filetype: str, utils: CreateUtilsContent, exercise: CreateExerciseContent, executor: Executor) -> None:
        self.name = name
        self.filetype = filetype
        self.utils_content = utils
        self.exercise_content = exercise
        self.executor = executor
        self.register_language()

    def register_language(self):
        LANGUAGES[self.name] = self

LANGUAGES: dict[str, Language] = {}

def get_language(language: Optional[str]) -> Language:
    if language is None:
        logger.warning(messages.DEFAULT_LANGUAGE_MESSAGE)
        return LANGUAGES['python']

    if language not in LANGUAGES:
        raise NotImplementedError(messages.LANGUAGE_NOT_FOUND_MESSAGE.format(language=language, 
                                                                             supported=LANGUAGES.keys()))
    return LANGUAGES[language]

def log_executor(executor: Executor):
    def wrapper(*args, **kwargs):
        logger.info(messages.EXECTOR_RUN_CMD.format(execuctor=executor.__name__))
        start_time = time()
        output = executor(*args, **kwargs)
        run_time = time() - start_time
        logger.info(messages.EXECTUTOR_FINISHED_CMD.format(seoncds=run_time, output=output))
        return output
    return wrapper
