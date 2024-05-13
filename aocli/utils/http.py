import logging
from urllib.parse import urlencode
from urllib.request import Request, urlopen

from aocli.utils import messages
from aocli.utils.session import get_aoc_session

logger = logging.getLogger(__file__)


class AocURL:
    def __init__(self) -> None:
        self.url = "https://adventofcode.com"

    def puzzle(self, year: int, day: int):
        return self.url + f"/{year}/day/{day}"

    def input(self, year: int, day: int):
        return self.puzzle(year=year, day=day) + "/input"

    def submit(self, year: int, day: int):
        return self.puzzle(year=year, day=day) + "/answer"

    def calendar(self, year):
        return self.url + f"/{year}"


class AocHttp:
    def __init__(self, session: str = get_aoc_session()) -> None:
        self.url = AocURL()
        self.headers = {"Cookie": f"session={session}"}

    def get_puzzle(self, year: int, day: int) -> str:
        return self._get(self.url.puzzle(year=year, day=day))

    def get_input(self, year: int, day: int) -> str:
        return self._get(self.url.input(year=year, day=day))

    def get_calendar(self, year: int) -> str:
        return self._get(self.url.calendar(year=year))

    def _get(self, url: str) -> str:
        request = Request(url, headers=self.headers)
        response = self._send_request(request)
        logger.debug(messages.DEBUG_RAW_GET_RESPONSE.format(response=response))
        return response

    def post_answer(self, year: int, day: int, answer: int, exercise: int) -> str:
        data = urlencode({"level": exercise, "answer": answer}).encode()
        return self._post(self.url.submit(year=year, day=day), data=data)

    def _post(self, url: str, data: bytes) -> str:
        request = Request(url, data=data, headers=self.headers)
        response = self._send_request(request)
        logger.debug(messages.DEBUG_RAW_POST_RESPONSE.format(response=response))
        return response

    def _send_request(self, request: Request) -> str:
        with urlopen(request) as response:
            response = response.read().decode("utf-8")
        return response
