from enum import StrEnum


class LanguageName(StrEnum):
    PYTHON = "python"
    RUST = "rust"


class InputType(StrEnum):
    EXAMPLE = "example"
    INPUT = "input"


class Command(StrEnum):
    START = "start"
    OPEN = "open"
    FETCH = "fetch"
    RUN = "run"
    CALENDAR = "calendar"
