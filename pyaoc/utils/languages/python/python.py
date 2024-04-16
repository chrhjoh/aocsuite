from pyaoc.utils.languages.python.utils import create_python_utils
from pyaoc.utils.languages.python.exercise import create_python_exercise
from pyaoc.utils.languages.language_factory import Language
from pyaoc.utils.languages.python.executor import python_executor

Language(name = 'python',
         filetype = '.py',
         utils = create_python_utils,
         exercise = create_python_exercise,
         executor = python_executor)
    
