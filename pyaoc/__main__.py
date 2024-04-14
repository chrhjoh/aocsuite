from pyaoc.utils.parsing import parse_args
from pyaoc.commands.fetch import get_and_save_experiment_files
from pyaoc.templates.initialize import create_experiment_from_template
from pathlib import Path
import sys
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

    EXPERIMENT_DIR = Path().cwd() / str(args.year) / str(args.day)

    if not EXPERIMENT_DIR.exists():
        logger.info(f'AOC Solution directory {EXPERIMENT_DIR} was not found. Initializing Directories and templates')

        create_experiment_from_template(str(EXPERIMENT_DIR), 'python')
        get_and_save_experiment_files(str(EXPERIMENT_DIR), args.year, args.day)
        logger.warning(f'Directory {EXPERIMENT_DIR} and templates were created. Please solve exercise 1 and then run and submit from here.')
        logger.warning('Exercise 2 can be fetched after solving exercise 1 by running with -f/--fetch argument.')
        sys.exit(0)

    if args.fetch:
        get_and_save_experiment_files(str(EXPERIMENT_DIR),args.year, args.day)

    if args.run and args.exercise:
       pass

    if args.submit and args.exercise:
        pass

if __name__ == '__main__':
    main()


