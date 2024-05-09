import logging
import os
import sys
from pathlib import Path

from pyaoc.commands.calendar import calendar
from pyaoc.commands.fetch import fetch
from pyaoc.commands.submit import submit
from pyaoc.utils import filenames, messages
from pyaoc.utils.io import File
from pyaoc.utils.languages import language_factory
from pyaoc.utils.parsing import InputType, parse_args

logging.basicConfig(
    datefmt="%Y-%m-%d %H:%M:%S",
    format="[%(asctime)s][%(module)s][%(levelname)s] %(message)s",
    level=logging.INFO,
    handlers=[
        logging.StreamHandler(),
    ],
)

logger = logging.getLogger(__file__)


def main():
    args = parse_args()

    if args.calendar:
        calendar(args.year)
        exit(0)

    if not args.day:
        raise ValueError("Day must be specified unless viewing calender")

    EXERCISE_DIR = (Path(args.base_dir) / str(args.year) / str(args.day)).resolve()

    language = language_factory.get_language(args.language, directory=str(EXERCISE_DIR))
    if not EXERCISE_DIR.exists():
        logger.info(messages.INITIALIZE_DIRECTORY.format(directory=EXERCISE_DIR))
        os.makedirs(EXERCISE_DIR)
        language.initialize()

        fetch(args.year, args.day, str(EXERCISE_DIR))
        logger.warning(
            messages.INITIALIZED_SUCCESS.format(
                directory=EXERCISE_DIR,
                year=args.year,
                day=args.day,
                exercise=args.exercise if args.exercise else 1,
            )
        )
        sys.exit(0)

    if args.fetch:
        fetch(args.year, args.day, str(EXERCISE_DIR))

    if args.run and args.exercise:
        for exercise in args.exercise:
            data_file = (
                filenames.EXAMPLE_FILE if args.input == InputType.EXAMPLE else filenames.INPUT_FILE
            )
            data_path = str(EXERCISE_DIR / data_file)

            if args.input == InputType.INPUT:
                answer_file = filenames.ANSWER_FILE.format(exercise=exercise)
                answer_path = str(EXERCISE_DIR / answer_file)
            else:
                answer_path = None

            language.execute(exercise, data_path, answer_path)

            if not args.no_submit and args.input == InputType.INPUT:
                answer_path = str(EXERCISE_DIR / filenames.ANSWER_FILE.format(exercise=exercise))
                answer = File.load(answer_path).content
                response = submit(args.year, args.day, exercise, int(answer))
                print(f"Submission response from Advent of Code:\n{response}")


if __name__ == "__main__":
    main()
