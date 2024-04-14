import subprocess
from pathlib import Path
from pyaoc.commands.executor.factory import register_executor
import logging

logger = logging.getLogger(__file__)

@register_executor('python')
def execute(directory: str, exercise: int, data_path: str) -> int:
    exercise_file = Path(directory) / f'exercise{exercise}.py'
    cmd = ['python', exercise_file, '--data', data_path]

    logger.info(f'Running Command: {" ".join(cmd)}')
    output = subprocess.run(cmd, stdout=subprocess.PIPE)
    logger.info(f'Result from exercise: {output}')
    
    return int(output.stdout.strip())


