
def create_python_exercise()-> str:
    return """from .utils import parse_args, load_data

def exercise() -> int:
    args = parse_args()
    data = load_data(args.data_path)

    return 0

if __name__ == '__main__':
    result = exercise()
    print(result)

    """
