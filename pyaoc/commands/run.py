from typing import Optional
from pyaoc.utils.languages.language_factory import Executor
from pyaoc.utils.io import File
from pyaoc.utils.filenames import ANSWER_FILE

def run_executor(executor: Executor, exercise_path: str, data_path:str, answer_path: Optional[str]) -> File:
    output = executor(exercise_path, data_path, answer_path)
    return File(ANSWER_FILE, str(output))

