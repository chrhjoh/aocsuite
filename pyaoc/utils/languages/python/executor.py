import subprocess
from typing import Optional
from pyaoc.utils.languages.language_factory import log_executor

@log_executor
def executor(exercise_path: str, input_path: str, answer_path: Optional[str]) -> int: 
    cmd = ['python', str(exercise_path), '--data-path', input_path]
    if answer_path:
        cmd.extend(['--answer-path'])
    output = subprocess.run(cmd, stdout=subprocess.PIPE)

    return int(output.stdout.strip())
