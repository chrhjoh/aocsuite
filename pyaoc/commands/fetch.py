from html2text import html2text
from pyaoc.utils.session import get_aoc_session
from pyaoc.utils.url import get_puzzle_url, get_input_url
from pyaoc.utils.parsing import parse_html_tag
from pyaoc.utils.io import File
from urllib.request import Request, urlopen
from logging import getLogger
from typing import Tuple
from time import sleep

logger = getLogger(__file__)

def get_and_save_experiment_files(save_directory: str, year: int, day: int) -> None:
    session = get_aoc_session()
    puzzle_url = get_puzzle_url(year, day)
    input_url = get_input_url(year, day)

    logger.debug(f'AoC session: {session}')
    logger.debug(f'Puzzle url: {puzzle_url}')
    logger.debug(f'Input Data url: {input_url}')

    input_data = fetch_input_data(input_url, session)
    sleep(5)
    description, example = fetch_description_in_markdown(puzzle_url, session)
    
    logger.debug(f'Sample from input data: {input_data[:200]}')
    logger.debug(f'Sample from description: {description[:200]}')
    logger.debug(f'Sample from example input: {example[:200]}')

    for content, name in zip([input_data, description, example], ['input.txt', 'description.md', 'example.txt']):
        File(name=save_directory + f'/{name}', content=content).save()

    logger.info(f'Successfully fetched input data and descriptions to {save_directory}')

def fetch_description_in_markdown(url: str, session)->Tuple[str, str]:
    req = Request(url, headers={"Cookie": f'session={session}'})
    with urlopen(req) as response:
        description_html = response.read().decode('utf-8')
    import pdb;pdb.set_trace()
    description_html = parse_html_tag(description_html, 'article', False)
    description = html2text(description_html).strip()
    example = parse_html_tag(description_html, 'code', True).strip()
    return description, example


def fetch_input_data(url: str, session: str) -> str:
    req = Request(url, headers={"Cookie": f'session={session}'})
    with urlopen(req) as response:
        input_data = response.read().decode('utf-8').strip()
    return input_data

