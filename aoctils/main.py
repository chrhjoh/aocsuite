import logging
import os
import subprocess
import sys

from aoctils.aoc_client import AocClient
from aoctils.aoc_directory import AocDataDirectory
from aoctils.languages.factory import LanguageAdapter, get_language
from aoctils.utils import enums, filenames
from aoctils.utils.parsing import (
    parse_args,
    puzzle_has_released,
    valid_calendar_request,
)

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
    # TODO: Fix how the Language loads its template. init should use language and create the base template for each language. new command should create a new exercise. It should fail if init has not been run first. Open should fail if new has not been run. New should also run open. Fetch should only fetch from language.
    aoc_client = AocClient(year=args.year, day=args.day)

    if args.command == enums.Command.CALENDAR:
        if valid_calendar_request(year=args.year):
            aoc_client.calendar()
            sys.exit(0)
        else:
            raise ValueError(f"Advent of Code has not started for year {args.year}")

    language = get_language(
        args.language, base_dir=args.base_dir, day=args.day, year=args.year
    )
    data_directory = AocDataDirectory(
        os.path.join(args.base_dir, "data"), args.year, args.day, force=args.force
    )
    os.makedirs(data_directory.base_dir, exist_ok=True)

    if (
        not puzzle_has_released(year=args.year, day=args.day)
        and not args.command == enums.Command.INIT
    ):
        raise ValueError(
            "The specified date and year is not a valid exercise, Puzzles are released at midnight EST"
        )

    if not data_directory.exists():
        data_directory.initialize()

    should_update_puzzle = False
    match args.command:
        case enums.Command.INIT:
            data_directory.initialize()
            language.apply_base_template()

        case enums.Command.OPEN:
            puzzle_path, example_path, exercise_path = get_files_for_editor(
                data_directory, language
            )
            open_editor(
                puzzle_file=puzzle_path,
                example_file=example_path,
                exercise_file=exercise_path,
            )

        case enums.Command.NEW:
            aoc_client.download(data_directory=data_directory)
            language.apply_exercise_template()

        case enums.Command.DOWNLOAD:
            aoc_client.download(data_directory=data_directory)

        case enums.Command.TEMPLATE:
            language.apply_base_template()
            language.apply_exercise_template()

        case enums.Command.RUN:
            if not language.is_initialized():
                raise ValueError("Exercise files were not intialized")
            if not data_directory.is_initialized():
                raise ValueError("Data files were not intialized")

            data_path = (
                data_directory / filenames.EXAMPLE_FILE
                if args.input == enums.InputType.EXAMPLE
                else data_directory / filenames.INPUT_FILE
            )

            language(args.exercise, str(data_path))
            if not args.no_submit and args.input == enums.InputType.INPUT:
                should_update_puzzle = submit(aoc_client, args.exercise)

        case enums.Command.SUBMIT:
            should_update_puzzle = submit(aoc_client, args.exercise)

    if should_update_puzzle:
        aoc_client.update_puzzle(data_directory)


def submit(aoc_client: AocClient, exercise: int):
    answer = int(input("Please input answer: "))
    response = aoc_client.submit(exercise, answer)
    print(f"Submission response from Advent of Code:\n{response}")
    if exercise == 1 and "right answer" in response.lower():
        print("Fetching exercise 2 from Advent of Code")


def get_files_for_editor(data_directory: AocDataDirectory, language: LanguageAdapter):
    puzzle_path = str(data_directory / filenames.PUZZLE_FILE)
    example_path = str(data_directory / filenames.EXAMPLE_FILE)
    exercise_path = language.get_exercise_path()
    return puzzle_path, example_path, exercise_path


def open_editor(
    puzzle_file: str,
    example_file: str,
    exercise_file: str,
    editor: str = os.environ.get("EDITOR", "NA"),
) -> None:
    if editor.lower() == "nvim":
        subprocess.run(
            [
                "nvim",
                exercise_file,
                f"+vsplit {example_file}",
                f"+split {puzzle_file}",
            ]
        )
    else:
        print("Open is not inplemented for you editor")


if __name__ == "__main__":
    main()
