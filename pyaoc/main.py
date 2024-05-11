import logging
import os
import sys
from pathlib import Path

from pyaoc.commands.calendar import calendar
from pyaoc.commands.fetch import fetch
from pyaoc.commands.submit import submit
from pyaoc.languages import factory as language_factory
from pyaoc.utils import enums, filenames, messages
from pyaoc.utils.io import File, verify_initialization
from pyaoc.utils.parsing import parse_args

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
    if args.command == enums.Command.CALENDAR:
        calendar(args.year)
        sys.exit(0)

    EXERCISE_DIR = (Path(args.base_dir) / str(args.year) / str(args.day)).resolve()
    if not EXERCISE_DIR.exists() and not args.command == enums.Command.INIT:
        raise FileNotFoundError(
            f"Directory {EXERCISE_DIR} was not found. You should run init to create the exercise directory?"
        )

    language = language_factory.get_language(args.language, directory=str(EXERCISE_DIR))

    match args.command:
        case enums.Command.INIT:
            if (
                verify_initialization(
                    EXERCISE_DIR,
                    f"Directory {EXERCISE_DIR} already exists. Do you want to overwrite it? [y/n]",
                )
                or args.force
            ):
                logger.info(
                    messages.INITIALIZE_DIRECTORY.format(directory=EXERCISE_DIR)
                )
                os.makedirs(EXERCISE_DIR, exist_ok=True)
                language.initialize()

                fetch(args.year, args.day, str(EXERCISE_DIR), args.force)
                logger.info(
                    messages.INITIALIZED_SUCCESS.format(
                        directory=EXERCISE_DIR,
                        year=args.year,
                        day=args.day,
                        exercise=(
                            args.exercise[0]
                            if isinstance(args.exercise, list)
                            else 1
                            if args.exercise is None
                            else args.exercise
                        ),
                    )
                )

        case enums.Command.FETCH:
            fetch(args.year, args.day, str(EXERCISE_DIR), args.force)

        case enums.Command.RUN:
            data_file = (
                filenames.EXAMPLE_FILE
                if args.input == enums.InputType.EXAMPLE
                else filenames.INPUT_FILE
            )
            data_path = str(EXERCISE_DIR / data_file)

            if args.input == enums.InputType.INPUT:
                answer_file = filenames.ANSWER_FILE.format(exercise=args.exercise)
                answer_path = str(EXERCISE_DIR / answer_file)
            else:
                answer_path = None

            language(args.exercise, data_path, answer_path)

            if not args.no_submit and args.input == enums.InputType.INPUT:
                answer_path = str(
                    EXERCISE_DIR / filenames.ANSWER_FILE.format(exercise=args.exercise)
                )
                answer = File.load(answer_path).content
                response = submit(args.year, args.day, args.exercise, int(answer))
                print(f"Submission response from Advent of Code:\n{response}")


if __name__ == "__main__":
    main()
