from pyaoc.utils.parsing import parse_args
from pyaoc.commands.fetch import get_and_save_experiment_files
from pyaoc.templates.factory import create_language_template
from pyaoc.commands.executor.factory import get_executor
from pyaoc.commands.submit import submit
from pyaoc.utils.io import File, load_answer
from pathlib import Path
import sys
import os
import logging

logging.basicConfig(datefmt="%Y-%m-%d %H:%M:%S",
        format="[%(asctime)s][%(module)s][%(levelname)s] %(message)s",
        level=logging.DEBUG,
        handlers=[
            logging.StreamHandler(),
        ]
        )

logger = logging.getLogger(__file__)

def main():
    args = parse_args()
    language = 'python'
    EXPERIMENT_DIR = Path().cwd() / str(args.year) / str(args.day)

    if not EXPERIMENT_DIR.exists():
        logger.info(f'AOC Solution directory {EXPERIMENT_DIR} was not found. Initializing Directories and templates')
        os.makedirs(EXPERIMENT_DIR)
        create_language_template(str(EXPERIMENT_DIR),language)
        get_and_save_experiment_files(str(EXPERIMENT_DIR), args.year, args.day)
        logger.warning(f'Directory {EXPERIMENT_DIR} and templates were created. Please solve exercise 1 and then run and submit from here.')
        logger.warning('Exercise 2 can be fetched after solving exercise 1 by running with -f/--fetch argument.')
        sys.exit(0)

    if args.fetch:
        get_and_save_experiment_files(str(EXPERIMENT_DIR),args.year, args.day)

    if args.run and args.exercise:
        executor = get_executor(language)
        output = executor(str(EXPERIMENT_DIR), args.exercise, args.input)
        File(name=str(EXPERIMENT_DIR / f'answer{args.exercise}.txt'),
             content=str(output)).save()

    if args.submit and args.exercise:
        answer = load_answer(str(EXPERIMENT_DIR), args.exercise)
        response = submit(args.year, args.day, args.exercise, answer)
        print(response)

if __name__ == '__main__':
    main()

