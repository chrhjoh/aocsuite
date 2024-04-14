BASE_URL = 'https://adventofcode.com'


def get_puzzle_url(year, day):
    return BASE_URL + f"/{year}/day/{day}"


def get_input_url(year, day):
    return get_puzzle_url(year, day) + '/input'

def get_submit_url(year, day):
    return get_puzzle_url(year, day) + '/answer'
