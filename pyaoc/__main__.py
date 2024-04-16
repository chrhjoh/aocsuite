from pyaoc.utils.parsing import parse_args
from pyaoc.commands.fetch import fetch_all_files, fetch_exercise_files
from pyaoc.commands.run import run_executor
from pyaoc.commands.calendar import calendar
from pyaoc.utils import filenames, messages
from pyaoc.utils.languages import language_factory
from pyaoc.commands.submit import submit
from pyaoc.utils.io import AocDirectory
import sys
import os
import logging

logging.basicConfig(datefmt="%Y-%m-%d %H:%M:%S",
        format="[%(asctime)s][%(module)s][%(levelname)s] %(message)s",
        level=logging.INFO,
        handlers=[
            logging.StreamHandler(),
        ]
        )

logger = logging.getLogger(__file__)

def main():
    args = parse_args()

    if args.calendar:
        calendar(args.year)
        exit(0)
    
    if not args.day:
        raise ValueError('Day must be specified unless viewing calender')

    language = language_factory.get_language('python')
    DAY_DIRECTORY = AocDirectory(args.base_dir, year=args.year, day=args.day)

    if not DAY_DIRECTORY.exists():
        logger.info(messages.INITIALIZE_DIRECTORY.format(directory=DAY_DIRECTORY.directory))
        os.makedirs(DAY_DIRECTORY.directory)
        
        all_files = fetch_all_files(args.year, args.day, language)
        DAY_DIRECTORY.save_files_in_dir(all_files)
        logger.warning(messages.INITIALIZED_SUCCESS.format(directory=DAY_DIRECTORY, 
                                                           year=args.year, 
                                                           day=args.day, 
                                                           exercise=args.exercise if args.exercise else 1))
        sys.exit(0)

    if args.fetch:
        fetched_files = fetch_exercise_files(args.year, args.day)
        DAY_DIRECTORY.save_files_in_dir(fetched_files, skip_if_exists=[filenames.EXAMPLE_FILE])

    if args.run and args.exercise:
        exercise_file = filenames.EXERCISE_FILE.format(exercise=args.exercise, filetype=language.filetype)
        exercise_path = DAY_DIRECTORY.path_in_directory(exercise_file)
        data_file = filenames.EXAMPLE_FILE if args.input == 'example' else filenames.INPUT_FILE
        data_path = DAY_DIRECTORY.path_in_directory(data_file)
        
        if args.input != 'example':
            answer_file = filenames.ANSWER_FILE.format(exercise=args.exercise)
            answer_path = DAY_DIRECTORY.path_in_directory(answer_file)
        else:
            answer_path = None

        run_executor(language.executor, exercise_path, data_path, answer_path)

    if args.submit and args.exercise:
        answer = DAY_DIRECTORY.load_file(filenames.ANSWER_FILE.format(exercise=1)).content
        response = submit(args.year, args.day, args.exercise, int(answer))
        print(f'Submission response from Advent of Code:\n{response}')

if __name__ == '__main__':
    main()

