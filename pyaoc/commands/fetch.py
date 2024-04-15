from typing import Iterable
from html2text import html2text
from pyaoc.utils.http import initialize_http
from pyaoc.utils.languages.language_factory import Language
from pyaoc.utils.parsing import parse_html_tag
from pyaoc.utils import messages
from pyaoc.utils import filenames
from logging import getLogger
from time import sleep
from pyaoc.utils.io import File

logger = getLogger(__file__)

def fetch_exercise_files(year: int, day: int) -> Iterable[File]:
    http = initialize_http(year=year, day=day)
    input_data = http.get_input()
    sleep(2)
    raw_puzzle = http.get_puzzle()
    puzzle = parse_puzzle_to_markdown(raw_puzzle)
    example = parse_puzzle_to_example(raw_puzzle)
    
    files = File(filenames.PUZZLE_FILE, puzzle), File(filenames.INPUT_FILE, input_data), File(filenames.EXAMPLE_FILE, example)

    for file in files:
         logger.debug(messages.DEBUG_PARSED_AOC_FILES.format(input_type=file.name, sample=file.content[:200]))

    return files

def fetch_language_files(language: Language) -> Iterable[File]:
    files =[
        File(filenames.EXERCISE_FILE.format(exercise=1, filetype=language.filetype), language.exercise_content()),
        File(filenames.EXERCISE_FILE.format(exercise=2, filetype=language.filetype), language.exercise_content()),
        File(filenames.UTIL_FILE.format(filetype=language.filetype), language.utils_content())
    ]
    return files

def fetch_all_files(year: int, day: int, language: Language) -> Iterable[File]:
    files = []
    files.extend(fetch_exercise_files(year, day))
    files.extend(fetch_language_files(language))
    return files
    

def parse_puzzle_to_markdown(raw_puzzle: str) -> str:
    article = parse_article(raw_puzzle)
    markdown = html2text(article).strip()
    return markdown

def parse_puzzle_to_example(raw_puzzle: str) -> str:
    article = parse_article(raw_puzzle)
    example = parse_html_tag(article, 'code', True).strip()
    return example

def parse_article(puzzle: str) -> str:
    return parse_html_tag(puzzle, 'article', False)

