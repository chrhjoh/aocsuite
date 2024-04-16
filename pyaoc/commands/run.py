from typing import Optional
from pyaoc.utils.languages.language_factory import Executor

def run_executor(executor: Executor, exercise_path: str, data_path:str, answer_path: Optional[str]):
    executor(exercise_path, data_path, answer_path)

