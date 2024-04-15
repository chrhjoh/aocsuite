from typing import Optional
from urllib.request import Request, urlopen
from urllib.parse import urlencode
from pyaoc.utils.session import get_aoc_session
from pyaoc.utils import messages
import logging
BASE_URL = 'https://adventofcode.com'

logger = logging.getLogger(__file__)

class AocURL():
    def __init__(self, year: Optional[int] = None, day: Optional[int] = None, url: str = BASE_URL) -> None:
        self.url = url
        self.year = year
        self.day = day
    @property
    def puzzle(self):
        if not self.year and self.day:
            raise ValueError('Must specify both year and date to get puzzle')
        return self.url + f'/{self.year}/day/{self.day}'
    @property
    def input(self):
        return self.puzzle  + '/input'
    @property
    def submit(self):
        return self.puzzle + '/answer'
    @property
    def calendar(self):
        return self.url + f'/{self.year}'

class AocRequest():

    def __init__(self, year: Optional[int] = None, day: Optional[int]=None,  session : str = get_aoc_session()) -> None:
        self.url = AocURL(year=year, day=day)
        self.headers = {"Cookie": f'session={session}'}

    def get_puzzle(self) -> str:
        return self._get(self.url.puzzle)

    def get_input(self) -> str:
        return self._get(self.url.input)
    
    def get_calendar(self) -> str:
        return self._get(self.url.calendar)

    def _get(self, url: str) -> str:
        request = Request(url, headers=self.headers)
        response = self._send_request(request)
        logger.debug(messages.DEBUG_RAW_GET_RESPONSE.format(response=response))
        return response

    def post_answer(self, answer: int, exercise: int) -> str:
        data = urlencode({'level' : exercise, 'answer' : answer}).encode()
        return self._post(self.url.submit, data=data)

    def _post(self, url: str, data: bytes) -> str:
        request = Request(url, data=data, headers=self.headers)
        response = self._send_request(request)
        logger.debug(messages.DEBUG_RAW_POST_RESPONSE.format(response=response))
        return response
        
    def _send_request(self, request: Request) -> str:
        with urlopen(request) as response:
            response = response.read().decode('utf-8')
        return response


def initialize_http(year: Optional[int] = None, day: Optional[int] = None) -> AocRequest:
    return AocRequest(year, day)

    

