import logging
import re
from typing import Tuple

from bs4 import BeautifulSoup
from html2text import html2text

from aocli.aoc_directory import AocDirectory
from aocli.utils import filenames, messages
from aocli.utils.http import AocHttp
from aocli.utils.parsing import parse_html_tag

logger = logging.getLogger(__file__)


class AocClient:
    def __init__(self, year: int, day: int) -> None:
        self.parser = AocParser()
        self.http = AocHttp()
        self.year = year
        self.day = day

    def submit(self, exercise: int, answer: int) -> str:
        response = self.http.post_answer(
            year=self.year, day=self.day, answer=answer, exercise=exercise
        )
        response = parse_html_tag(response, "article", True)
        response = response[: response.find("[")]

        logger.debug(messages.DEBUG_PARSED_SUBMIT_RESPONSE.format(response=response))

        return response

    def calendar(self):
        calendar = self.http.get_calendar(self.year)
        calendar = self.parser.parse_calendar(calendar)
        print(calendar)

    def fetch(self, directory: AocDirectory) -> None:
        input = self.http.get_input(year=self.year, day=self.day)
        raw_puzzle = self.http.get_puzzle(year=self.year, day=self.day)
        puzzle, example = self.parser.parse_puzzle(raw_puzzle)
        directory.save_files(
            {
                filenames.INPUT_FILE: input,
                filenames.EXAMPLE_FILE: example,
                filenames.PUZZLE_FILE: puzzle,
            }
        )

    def get_example_name(self):
        return filenames.EXAMPLE_FILE

    def get_puzzle_name(self):
        return filenames.PUZZLE_FILE


class AocParser:
    def _parse_puzzle_to_markdown(self, raw_puzzle: str) -> str:
        article = self.parse_article(raw_puzzle)
        markdown = html2text(article).strip()
        return markdown

    def _parse_puzzle_to_example(self, raw_puzzle: str) -> str:
        article = self.parse_article(raw_puzzle)
        example = parse_html_tag(article, "pre", False)
        example = parse_html_tag(example, "code", True).strip()
        return example

    def parse_puzzle(self, raw_puzzle: str) -> Tuple[str, str]:
        puzzle = self._parse_puzzle_to_markdown(raw_puzzle)
        example = self._parse_puzzle_to_example(raw_puzzle)
        return puzzle, example

    def parse_article(self, puzzle: str) -> str:
        return parse_html_tag(puzzle, "article", False)

    def _pad_calendar(self, calendar: str):
        list_calendar = calendar.split("\n")
        padded_calendar = []
        max_length = max([len(line) + 1 for line in list_calendar])
        max_stars = max(
            [
                line.strip().count("*", len(line.strip()) - 3, len(line.strip()))
                for line in list_calendar
            ]
        )
        for line in calendar.split("\n"):
            length_diff = max_length - len(line)
            stars_diff = max_stars - line.strip().count(
                "*", len(line.strip()) - 3, len(line.strip())
            )
            line = " " * (length_diff - stars_diff + 1) + line
            padded_calendar.append(line)
        return "\n".join(padded_calendar)

    def parse_calendar(self, calendar: str) -> str:
        calendar = parse_html_tag(
            calendar, ["span", "a"], False, class_=re.compile("calendar")
        )
        calendar = self._parse_calendar_stars(calendar)
        calendar = self._pad_calendar(calendar)

        return calendar

    def _parse_calendar_stars(self, calendar: str) -> str:
        soup = BeautifulSoup(calendar, "html.parser")
        parsed_calendar_lines = []
        for day in soup.find_all("a"):
            try:
                if not "two stars" in day["aria-label"].lower():
                    day.find("span", class_="calendar-mark-verycomplete").decompose()
                if (
                    not "one star" in day["aria-label"].lower()
                    and not "two stars" in day["aria-label"].lower()
                ):
                    day.find("span", class_="calendar-mark-complete").decompose()

                parsed_calendar_lines.append(str(day.text))
            except KeyError:
                parsed_calendar_lines.append(str(day.text))
        return "\n".join(parsed_calendar_lines)
