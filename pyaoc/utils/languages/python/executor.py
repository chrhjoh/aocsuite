import subprocess
from pyaoc.utils.languages.language_factory import log_executor

@log_executor
def executor(exercise_path: str, input_path: str) -> int: 
    cmd = ['python', str(exercise_path), '--data', input_path]
    output = subprocess.run(cmd, stdout=subprocess.PIPE)

    return int(output.stdout.strip())
