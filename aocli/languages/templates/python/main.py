from argparse import ArgumentParser, Namespace

from exercise import exercise1, exercise2


class AocNamespace(Namespace):
    data_path: str
    answer_path: str
    exercise = int


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
    parser.add_argument(
        "--exercise",
        type=int,
        action="store",
        help="Exercise to run.",
    )

    return parser.parse_args(namespace=AocNamespace())


def load_data(data_path: str) -> str:
    """
    Load Data from data_path to a string

    """
    return open(data_path, "r").read().strip()


def main():
    args = parse_args()
    input_data = load_data(args.data_path)

    if args.exercise == 1:
        answer = exercise1(input_data)
    elif args.exercise == 2:
        answer = exercise2(input_data)
    else:
        raise NotImplementedError()

    print(f"Answer to exercise {args.exercise}: {answer}")


if __name__ == "__main__":
    main()
