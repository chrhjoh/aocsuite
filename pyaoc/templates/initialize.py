import os    
from pyaoc.templates.languages import python_template
import logging

logger = logging.getLogger(__file__)

def create_experiment_from_template(directory: str, language: str) -> None:
    os.makedirs(directory)

    if language == 'python':
        exercise_template, shared_template = python_template.get_template()
        filetype='.py'
    else:
        raise NotImplementedError()
    for i in range(1,3):
        open(directory+'/exercise'+str(i)+filetype, 'w').write(exercise_template)
    open(directory+'/shared'+filetype, 'w').write(shared_template)

    logger.info(f'Successfully created Templates for python in directory: {directory} ')

