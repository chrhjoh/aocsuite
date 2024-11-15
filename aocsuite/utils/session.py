import os

SESSION_COOKIE_ENV = "ADVENT_OF_CODE_SESSION"
SESSION_COOKIE_FILE = "adventofcode.session"
SESSION_COOKIE_HIDDEN_FILE = ".adventofcode.session"


def get_aoc_session() -> str:
    if SESSION_COOKIE_ENV in os.environ:
        return os.environ[SESSION_COOKIE_ENV]
    elif os.path.exists(SESSION_COOKIE_FILE):
        return get_session_from_file(SESSION_COOKIE_FILE)
    elif os.path.exists(SESSION_COOKIE_HIDDEN_FILE):
        return get_session_from_file(SESSION_COOKIE_HIDDEN_FILE)

    raise FileNotFoundError(
        "Session could not be retrieved. Please specifiy file or environment variable"
    )


def get_session_from_file(file: str) -> str:
    return open(file).read().strip()
