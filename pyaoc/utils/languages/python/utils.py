def create_python_utils() -> str:
    return """from argparse import ArgumentParser, Namespace

class AocNamespace(Namespace):
    data_path : str

def parse_args():
    parser = ArgumentParser('Advent of Code Exercise')
    parser.add_argument('--data-path', type=str, action='store', required=True, help='Path to file input for exercise. Input or Example')
    return parser.parse_args(namespace=AocNamespace())

def load_data(data_path: str) -> str:
    '''
    Load Data from data_path to a string

    '''
    return open(data_path,'r').read()
    
    """


