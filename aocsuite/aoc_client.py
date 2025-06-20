import logging
import re
from typing import Optional, Tuple

from bs4 import BeautifulSoup, NavigableString, Tag
from markdownify import MarkdownConverter

from aocsuite.aoc_directory import AocDataDirectory
from aocsuite.utils import filenames, messages
from aocsuite.utils.http import AocHttp

logger = logging.getLogger(__file__)


class AocClient:
    def __init__(self, year: int, day: int) -> None:
        self.http = AocHttp()
        self.year = year
        self.day = day

    def submit(self, exercise: int, answer: str) -> str:
        response = self.http.post_answer(
            year=self.year, day=self.day, answer=answer, exercise=exercise
        )
        response = parse_submission_response(response)

        logger.debug(messages.DEBUG_PARSED_SUBMIT_RESPONSE.format(response=response))

        return response

    def calendar(self):
        calendar = self.http.get_calendar(self.year)
        calendar = parse_calendar(calendar)
        print(calendar)

    def download(self, data_directory: AocDataDirectory) -> None:
        input = self.http.get_input(year=self.year, day=self.day)
        raw_puzzle = self.http.get_puzzle(year=self.year, day=self.day)
        puzzle, example = parse_puzzle(raw_puzzle)
        data_directory.save_files(
            {
                filenames.INPUT_FILE: input,
                filenames.EXAMPLE_FILE: example,
                filenames.PUZZLE_FILE: puzzle,
            }
        )

    def update_puzzle(self, directory: AocDataDirectory):
        raw_puzzle = self.http.get_puzzle(year=self.year, day=self.day)
        puzzle, _ = parse_puzzle(raw_puzzle)
        directory.save_files(
            {
                filenames.PUZZLE_FILE: puzzle,
            },
            force=True,
        )


def parse_submission_response(html: str):
    soup = BeautifulSoup(html, "html.parser")
    article = soup.find("article")
    if article is not None:
        article = article.text[: article.text.find(r"[")]
    else:
        article = ""
    return article


def parse_puzzle(html: str) -> Tuple[str, str]:
    soup = BeautifulSoup(html, "html.parser")
    articles = soup.find_all("article")
    md_converter = MarkdownConverter()
    puzzle = "\n".join([md_converter.convert_soup(article) for article in articles])
    example = "\n".join(
        [
            md_converter.convert_soup(tag)
            for article in articles
            for tag in article.find_all("pre")
        ]
    )
    return puzzle.strip(), example.strip()


def parse_calendar(html: str) -> str:
    soup = BeautifulSoup(html, "html.parser")
    content = soup.find("main")
    if content is None or isinstance(content, NavigableString):
        return ""

    color_mapper = parse_css_classes_to_colors(soup.find("style"))

    # Remove all script and style tags
    for script_or_style in content(["script", "style"]):
        script_or_style.decompose()

    # Fix color of default text
    for a in soup.find_all("a"):
        # Iterate through all children of the <a> tag
        for child in a.contents:
            if isinstance(child, NavigableString):  # Check if the child is a text node
                # Create a new <span> with the desired color class
                new_span = soup.new_tag("span", **{"class": "calendar-default-text"})
                new_span.string = child  # Set the text for the new <span>
                a.insert(
                    a.contents.index(child), new_span
                )  # Insert the new <span> before the text node
                a.contents.remove(child)  # Remove the original text node

    content = parse_calendar_stars(content)
    if content is None or isinstance(content, NavigableString):
        return ""

    # Convert all colors into ANSI for terminals
    replace_css_with_ansi_colors(content.find_all("span"), color_mapper)

    # Extract text from the main content
    text = content.get_text()
    text = text.replace(
        "." * 11, "."
    )  # HACK: removes the overlapping "." in 2024 day 14
    return text


def parse_css_classes_to_colors(style_tags: Optional[list[Tag]]) -> dict[str, str]:
    # default styles
    class_color_mapping = {
        "calendar-default-text": "#666666",
        "calendar-day": "#cccccc",
        "calendar-mark-complete": "#ffff66",
        "calendar-mark-verycomplete": "#ffff66",
    }
    if style_tags is not None:
        # Find all <style> tags in the HTML
        for style_tag in style_tags:
            css_content = style_tag.string

            if css_content:
                # Use regex to extract class names and their corresponding color values
                pattern = re.compile(
                    r"(\.calendar-color-\w+)\s*{[^}]*color:\s*(#[0-9a-fA-F]{3,6})[^}]*}"
                )
                matches = pattern.findall(css_content)

                # Populate the dictionary with class-color mappings
                for match in matches:
                    class_name = match[0].strip(".")
                    color = match[1]
                    class_color_mapping[class_name] = color

    return class_color_mapping


def parse_calendar_stars(content: Tag) -> Tag:
    for a_tag in content.find_all("a"):
        aria_label = a_tag.get("aria-label", "")

        if "star" not in aria_label:
            for star_span in a_tag.find_all(
                "span",
                class_=["calendar-mark-complete", "calendar-mark-verycomplete"],
            ):
                star_span.string = " "
        elif "one star" in aria_label:
            for star_span in a_tag.find_all(
                "span",
                class_=["calendar-mark-verycomplete"],
            ):
                star_span.string = " "

    return content


def hex_to_ansi(hex_color: str):
    # Convert hex color to RGB
    hex_color = hex_color.lstrip("#")
    try:
        rgb = tuple(int(hex_color[i : i + 2], 16) for i in (0, 2, 4))
    except ValueError:
        return 0

    # Approximate conversion to ANSI escape code (using 256 color mode)
    r, g, b = rgb
    if r == g == b:
        # Grayscale conversion
        if r < 8:
            return 16
        elif r > 248:
            return 231
        else:
            return round(((r - 8) / 247) * 24) + 232
    else:
        # Color cube conversion
        return (
            16
            + (36 * round(r / 255 * 5))
            + (6 * round(g / 255 * 5))
            + round(b / 255 * 5)
        )


def replace_css_with_ansi_colors(span_tags, color_mapper):
    for tag in span_tags:
        if tag.name == "span" and "class" in tag.attrs:
            css_class = " ".join(tag["class"])
            if css_class in color_mapper:
                hex_color = color_mapper[css_class]
                ansi_color_code = hex_to_ansi(hex_color)
                # Apply ANSI escape code for the foreground color
                tag.insert(0, f"\033[38;5;{ansi_color_code}m")
                # Append the reset ANSI code after the content
                tag.append("\033[0m")
        tag.replace_with(tag.get_text())
