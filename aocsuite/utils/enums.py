from enum import StrEnum


class LanguageName(StrEnum):
    PYTHON = "python"
    RUST = "rust"


class InputType(StrEnum):
    EXAMPLE = "example"
    INPUT = "input"


class Command(StrEnum):
    INIT = "init"
    NEW = "new"
    OPEN = "open"
    DOWNLOAD = "dowload"
    TEMPLATE = "template"
    RUN = "run"
    SUBMIT = "submit"
    CALENDAR = "calendar"
