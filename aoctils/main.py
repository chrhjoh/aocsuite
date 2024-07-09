import logging
import os
import subprocess
import sys

from aoctils.aoc_client import AocClient
from aoctils.aoc_directory import AocDirectory
from aoctils.utils import enums
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
    aoc_client = AocClient(year=args.year, day=args.day)

    if args.command == enums.Command.CALENDAR:
        if valid_calendar_request(year=args.year):
            aoc_client.calendar()
            sys.exit(0)
        else:
            raise ValueError(f"Advent of Code has not started for year {args.year}")

    directory = AocDirectory(args.base_dir, args.year, args.day, force=args.force)
    os.makedirs(directory.base_dir, exist_ok=True)

    if not puzzle_has_released(year=args.year, day=args.day):
        raise ValueError(
            "The specified date and year is not a valid exercise, Puzzles are released at midnight EST"
        )

    if not directory.exists():
        directory.initialize()

    match args.command:
        case enums.Command.START:
            aoc_client.fetch(directory=directory)
            puzzle_path = directory / aoc_client.get_puzzle_name()
            example_path = directory / aoc_client.get_example_name()

            open_editor(
                puzzle_file=str(puzzle_path),
                example_file=str(example_path),
                exercise_file=str(args.exercise_path),
            )

        case enums.Command.OPEN:
            puzzle_path = directory / aoc_client.get_puzzle_name()
            example_path = directory / aoc_client.get_example_name()
            open_editor(
                puzzle_file=str(puzzle_path),
                example_file=str(example_path),
                exercise_file=str(args.exercise_path),
            )

        case enums.Command.FETCH:
            aoc_client.fetch(directory=directory)

        case enums.Command.SUBMIT:
            answer = int(input("Please input answer: "))
            response = aoc_client.submit(args.exercise, answer)
            print(f"Submission response from Advent of Code:\n{response}")
            if args.exercise == 1 and "right answer" in response.lower():
                print("Fetching exercise 2 from Advent of Code")
                aoc_client.update_puzzle(directory)


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
