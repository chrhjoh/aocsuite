from argparse import ArgumentParser, Namespace
from typing import List

class AocNamespace(Namespace):
    day : int
    year : int
    fetch : bool
    exercises: List[int]
    run : bool
    submit : bool



def parse_args():
    parser = ArgumentParser('Advent Of Code API')
    parser.add_argument('-d', '--day', type=int, action='store', required=True, choices=range(1,26), help='Which day (1-25) are you working on?')
    parser.add_argument('-y', '--year', type=int, action='store', required=True, choices=range(2017, 2024), help='Which year (>=2015) are you working on?')
    parser.add_argument('-f', '--fetch', action='store_true', help='Fetch the question from Advent of code. Also used to initialize folders for the code')
    parser.add_argument('-e', '--exercises', action='store', type=int, nargs=2, default=[1,2], choices=[1,2], help='Exercise to run and submit (if specified)')
    parser.add_argument('-r', '--run', action='store_true', help='Run exercises specified by exercises argument')
    parser.add_argument('-s', '--submit', action='store_true', help='Submit the exercises to Advent of Code')

    config = parser.parse_args(namespace=AocNamespace())
    validate_config(config)
    return config
    


def validate_config(config: AocNamespace):
    if config.day < 1 or config.day > 25:
        return ValueError('Day must be between 1 and 25')

    if config.year < 2017 or config.year > 2023:
        return ValueError('Year must be between 2017 and 2023')

    

