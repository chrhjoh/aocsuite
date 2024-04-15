from argparse import ArgumentParser, Namespace
import re
from bs4 import BeautifulSoup
import os
from typing import Optional, List

class AocNamespace(Namespace):
    base_dir : str
    day : Optional[int]
    year : int
    fetch : bool
    calendar : bool
    exercise: int | None
    input: str
    run : bool
    submit : bool

def parse_args():
    parser = ArgumentParser('Advent Of Code API')
    parser.add_argument('-b', '--base-dir', type=str, action='store', default=os.getcwd(), help='Which day (1-25) are you working on?')
    parser.add_argument('-d', '--day', type=int, action='store',  choices=range(1,26), help='Which day (1-25) are you working on?')
    parser.add_argument('-y', '--year', type=int, action='store', required=True, choices=range(2015, 2024), help='Which year (>=2015) are you working on?')
    parser.add_argument('-f', '--fetch', action='store_true', help='Fetch the question from Advent of code. ')
    parser.add_argument('-c', '--calendar', action='store_true', help='Fetch the Calender for selected year. ')
    parser.add_argument('-i', '--input', action='store', default='input', type=str, choices=['input','example'], help='What input data should be used for run. Input or example. (default = input)')
    parser.add_argument('-e', '--exercise', action='store', type=int, choices=[1,2], help='Exercise to run and submit (if specified)')
    parser.add_argument('-r', '--run', action='store_true', help='Run exercises specified by exercises argument')
    parser.add_argument('-s', '--submit', action='store_true', help='Submit the exercises to Advent of Code')

    config = parser.parse_args(namespace=AocNamespace())
    return config
    


def parse_html_tag(html_string: str, tag_name: str | List[str], only_text : bool, **kwargs) -> str:
    soup = BeautifulSoup(html_string, 'html.parser')
    tags = soup.find_all(tag_name, **kwargs)
    markdown_contents = []
    for tag in tags:
        html_content = tag.text if only_text else str(tag)
        markdown_contents.append(html_content.strip())  # Strip any leading/trailing whitespace
    return '\n'.join(markdown_contents)

def parse_calendar(calendar: str) -> str:
    calendar = parse_html_tag(calendar,["span", 'a'], False, class_=re.compile('calendar'))
    calendar = parse_calendar_stars(calendar)
    return calendar

def parse_calendar_stars(calendar: str) -> str:
    soup = BeautifulSoup(calendar, 'html.parser')
    parsed_calendar_lines = []
    for day in soup.find_all('a'):
        try:
            if not 'two stars' in day['aria-label'].lower():
                day.find('span', class_='calendar-mark-verycomplete').decompose()
            if not 'one star' in day['aria-label'].lower() and not 'two stars' in day['aria-label'].lower():
                day.find('span', class_='calendar-mark-complete').decompose()

            parsed_calendar_lines.append(str(day.text))
        except KeyError:
            parsed_calendar_lines.append(str(day.text))
    return '\n'.join(parsed_calendar_lines)
