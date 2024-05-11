from logging import getLogger
from pathlib import Path
from time import sleep

from html2text import html2text

from pyaoc.utils import filenames, messages
from pyaoc.utils.http import initialize_http
from pyaoc.utils.io import File, verify_initialization
from pyaoc.utils.parsing import parse_html_tag

logger = getLogger(__file__)


def fetch(year: int, day: int, directory: str, force: bool) -> None:
    http = initialize_http(year=year, day=day)
    input_data = http.get_input()
    sleep(2)
    raw_puzzle = http.get_puzzle()
    puzzle = parse_puzzle_to_markdown(raw_puzzle)
    example = parse_puzzle_to_example(raw_puzzle)

    files = [
        File(str(Path(directory) / filenames.PUZZLE_FILE), puzzle),
        File(str(Path(directory) / filenames.INPUT_FILE), input_data),
    ]
    example_path = Path(directory) / filenames.EXAMPLE_FILE
    if (
        verify_initialization(
            example_path,
            f"File {example_path} already exists. Do you want to overwrite it? You might have made manual examples in it? [y/n]. ",
        )
        or force
    ):
        files.append(File(str(Path(directory) / filenames.EXAMPLE_FILE), example))

    for file in files:
        logger.debug(
            messages.DEBUG_PARSED_AOC_FILES.format(
                input_type=file.name, sample=file.content[:200]
            )
        )
        file.save()


def parse_puzzle_to_markdown(raw_puzzle: str) -> str:
    article = parse_article(raw_puzzle)
    markdown = html2text(article).strip()
    return markdown


def parse_puzzle_to_example(raw_puzzle: str) -> str:
    article = parse_article(raw_puzzle)
    example = parse_html_tag(article, "code", True).strip()
    return example


def parse_article(puzzle: str) -> str:
    return parse_html_tag(puzzle, "article", False)
