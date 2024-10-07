from argparse import ArgumentParser, Namespace
from typing import Callable


class AocNamespace(Namespace):
    data_path: str
    answer_path: str
    exercise: int
    year: int
    day: int


def parse_args():
    parser = ArgumentParser("Advent of Code Exercise")
    parser.add_argument(
        "--data-path",
        type=str,
        action="store",
        required=True,
        help="Path to file input for exercise. Input or Example",
    )
    parser.add_argument(
        "--exercise",
        type=int,
        action="store",
        choices=[1, 2],
        help="Exercise to run.",
    )

    return parser.parse_args(namespace=AocNamespace())


def load_data(data_path: str) -> str:
    """
    Load Data from data_path to a string

    """
    return open(data_path, "r").read().strip()


def run(exercise1: Callable[[str], int], exercise2: Callable[[str], int]):
    args = parse_args()
    input_data = load_data(args.data_path)
    match args.exercise:
        case 1:
            exercise = exercise1
        case 2:
            exercise = exercise2
        case _:
            raise ValueError("Exercise cannot be more than 1 or 2")
    answer = exercise(input_data)
    print(f"Answer to exercise {args.exercise}: {answer}")
