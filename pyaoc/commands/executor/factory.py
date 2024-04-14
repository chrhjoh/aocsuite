from typing import Callable

EXECUTORFUNC = Callable[[str, int, str], int]

EXECUTOR_REGISTER: dict[str, EXECUTORFUNC] = {}

def register_executor(language: str) -> Callable:
    def wrapper(func: EXECUTORFUNC):
        EXECUTOR_REGISTER[language] = func

    return wrapper

def get_executor(language) -> EXECUTORFUNC:
    if language not in EXECUTOR_REGISTER:
        raise NotImplementedError(f'Language {language} is not implemented. Please use one of {EXECUTOR_REGISTER.keys()}')

    return EXECUTOR_REGISTER[language]
