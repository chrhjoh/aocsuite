import os
import re
from argparse import Action, ArgumentParser, Namespace
from enum import Enum
from typing import List

from bs4 import BeautifulSoup

from pyaoc.utils import enums


class AocNamespace(Namespace):
    command: enums.Command
    base_dir: str
    day: int
    year: int
    exercise: int
    input: enums.InputType
    no_submit: bool
    language: enums.LanguageName


class EnumAction(Action):
    """
    Argparse action for handling Enums
    """

    def __init__(self, **kwargs):
        # Pop off the type value
        enum_type = kwargs.pop("type", None)

        # Ensure an Enum subclass is provided
        if enum_type is None:
            raise ValueError("type must be assigned an Enum when using EnumAction")
        if not issubclass(enum_type, Enum):
            raise TypeError("type must be an Enum when using EnumAction")

        # Generate choices from the Enum
        kwargs.setdefault("choices", tuple(e.value for e in enum_type))

        super(EnumAction, self).__init__(**kwargs)

        self._enum = enum_type

    def __call__(self, parser, namespace, values, option_string=None):
        # Convert value back into an Enum
        value = self._enum(values)
        setattr(namespace, self.dest, value)


def parse_args():
    parser = ArgumentParser("Advent Of Code API")

    parser.add_argument(
        "command",
        action=EnumAction,
        type=enums.Command,
        help="""
        Possible commands:
            init: Initialize a directory with the selected language template. Also fetches the exercise descriptions and data from AoC.

            fetch: Fetches the exercise description and data to the directory. Useful when you want the second exercise of the day.

            run: Runs the exercises and submits to AoC if not using experiment data. Can be disabled with --no-submit

            calendar: Fetches the calendar for the specified year
            
        """,
    )

    parser.add_argument(
        "--language",
        type=enums.LanguageName,
        action=EnumAction,
        default=enums.LanguageName.PYTHON,
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
        default=None,
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
        action=EnumAction,
        default=enums.InputType.INPUT,
        type=enums.InputType,
        help="What input data should be used for run. Input or example. (default = input)",
    )
    parser.add_argument(
        "--exercise",
        action="store",
        type=int,
        choices=[1, 2],
        help="Exercise to run and submit (Depending on run and --no-submit). Multiple can be specified. Default is 1",
        default=1,
    )
    parser.add_argument(
        "--no-submit",
        action="store_true",
        help="Submit the exercises to Advent of Code. Also will not submit if input is example.",
    )

    config = parser.parse_args(namespace=AocNamespace())
    if not config.day and config.command != enums.Command.CALENDAR:
        raise ValueError("Day must be specified unless fetching calendar")

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
