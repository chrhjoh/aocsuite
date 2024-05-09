import os
import re
from argparse import ArgumentParser, Namespace
from enum import StrEnum
from typing import List, Optional

from bs4 import BeautifulSoup

from pyaoc.utils.languages.language_factory import Language


class InputType(StrEnum):
    EXAMPLE = "example"
    INPUT = "input"


class AocNamespace(Namespace):
    base_dir: str
    day: Optional[int]
    year: int
    fetch: bool
    calendar: bool
    exercise: List[int]
    input: InputType
    run: bool
    no_submit: bool
    language: Language


def parse_args():
    parser = ArgumentParser("Advent Of Code API")

    command_group = parser.add_mutually_exclusive_group(required=False)

    command_group.add_argument(
        "--fetch",
        action="store_true",
        required=False,
        help="Fetch the question from Advent of code. ",
    )
    command_group.add_argument(
        "--calendar",
        action="store_true",
        required=False,
        help="Fetch the Calender for selected year. ",
    )
    command_group.add_argument(
        "--run",
        action="store_true",
        required=False,
        help="Run exercises specified by exercises argument",
    )

    parser.add_argument(
        "--language",
        type=str,
        action="store",
        default="python",
        help="What language should be used for initialize and run (default = python)",
    )

    parser.add_argument(
        "--base-dir",
        type=str,
        action="store",
        default=os.path.relpath(os.getcwd()),
        help="What directory should be used as default directory (default = cwd)",
    )
    parser.add_argument(
        "--day",
        type=int,
        action="store",
        choices=range(1, 26),
        help="Which day (1-25) are you working on?",
    )
    parser.add_argument(
        "--year",
        type=int,
        action="store",
        required=True,
        choices=range(2015, 2024),
        help="Which year (>=2015) are you working on?",
    )

    parser.add_argument(
        "--input",
        action="store",
        default="input",
        type=str,
        choices=["input", "example"],
        help="What input data should be used for run. Input or example. (default = input)",
    )
    parser.add_argument(
        "--exercise",
        action="store",
        type=int,
        nargs="*",
        choices=[1, 2],
        help="Exercise to run and submit (Depending on run and --no-submit). Multiple can be specified. Default is 1",
        default=[1, 2],
    )
    parser.add_argument(
        "--no-submit",
        action="store_true",
        help="Submit the exercises to Advent of Code. Also will not submit if input is example.",
    )

    config = parser.parse_args(namespace=AocNamespace())
    config.input = InputType(config.input)
    config.language = Language(config.language)
    return config


def parse_html_tag(html_string: str, tag_name: str | List[str], only_text: bool, **kwargs) -> str:
    soup = BeautifulSoup(html_string, "html.parser")
    tags = soup.find_all(tag_name, **kwargs)
    markdown_contents = []
    for tag in tags:
        html_content = tag.text if only_text else str(tag)
        markdown_contents.append(html_content.strip())  # Strip any leading/trailing whitespace
    return "\n".join(markdown_contents)


def parse_calendar(calendar: str) -> str:
    calendar = parse_html_tag(calendar, ["span", "a"], False, class_=re.compile("calendar"))
    calendar = parse_calendar_stars(calendar)
    return calendar


def parse_calendar_stars(calendar: str) -> str:
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
