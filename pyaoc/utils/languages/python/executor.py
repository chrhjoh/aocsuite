import subprocess
from typing import Optional
from pyaoc.utils.languages.language_factory import log_executor
import logging

logger = logging.getLogger(__file__)

@log_executor
def python_executor(exercise_path: str, input_path: str, answer_path: Optional[str]) -> None: 
    cmd = ['python', str(exercise_path), '--data-path', input_path]
    if answer_path:
        cmd.extend(['--answer-path', answer_path])
    try:
        subprocess.run(cmd, check=True )
    except subprocess.CalledProcessError as err:
        logger.error(f'Error was encountered while running executor. Error Encountered:\n{err}')


