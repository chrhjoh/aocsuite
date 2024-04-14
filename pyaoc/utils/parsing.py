from argparse import ArgumentParser, Namespace

class AocNamespace(Namespace):
    day : int
    year : int
    fetch : bool
    exercise: int | None
    run : bool
    submit : bool



def parse_args():
    parser = ArgumentParser('Advent Of Code API')
    parser.add_argument('-d', '--day', type=int, action='store', required=True, choices=range(1,26), help='Which day (1-25) are you working on?')
    parser.add_argument('-y', '--year', type=int, action='store', required=True, choices=range(2017, 2024), help='Which year (>=2015) are you working on?')
    parser.add_argument('-f', '--fetch', action='store_true', help='Fetch the question from Advent of code. Also used to initialize folders for the code')
    parser.add_argument('-e', '--exercise', action='store', type=int, choices=[1,2], help='Exercise to run and submit (if specified)')
    parser.add_argument('-r', '--run', action='store_true', help='Run exercises specified by exercises argument')
    parser.add_argument('-s', '--submit', action='store_true', help='Submit the exercises to Advent of Code')

    config = parser.parse_args(namespace=AocNamespace())
    return config
    


    

