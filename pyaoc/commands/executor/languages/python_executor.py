import subprocess
from pathlib import Path
from pyaoc.commands.executor.factory import register_executor
import logging

logger = logging.getLogger(__file__)

@register_executor('python')
def execute(directory: str, exercise: int, input_type: str) -> int:
    exercise_path = Path(directory) / f'exercise{exercise}.py'
    data_file = 'input.txt' if input_type == 'input' else 'example.txt'
    data_path = Path(directory) / data_file
    cmd = ['python', str(exercise_path), '--data', str(data_path)]

    logger.info(f'Running Command: {" ".join(cmd)}')
    output = subprocess.run(cmd, stdout=subprocess.PIPE)
    logger.info(f'Result from exercise: {output}')
    
    return int(output.stdout.strip())


