from argparse import ArgumentParser, Namespace
from typing import Callable


class AocNamespace(Namespace):
    data_path: str
    answer_path: str


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
        "--answer-path",
        type=str,
        action="store",
        help="Path to save return value from exercise to.",
    )
    return parser.parse_args(namespace=AocNamespace())


def load_data(data_path: str) -> str:
    """
    Load Data from data_path to a string

    """
    return open(data_path, "r").read().strip()


def run_python_exercise(exercise_func: Callable[[str], int]):
    args = parse_args()
    input_data = load_data(args.data_path)
    answer = exercise_func(input_data)
    print(f"Answer to exercise: {answer}")
    if args.answer_path:
        print(f"Saving answer to {args.answer_path}.")
        open(args.answer_path, "w").write(str(answer))
    else:
        print("Not saving answer. Answer Path was not set. This was probably example data.")
