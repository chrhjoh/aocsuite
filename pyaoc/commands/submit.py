from urllib.request import Request, urlopen
from urllib.parse import urlencode
from pyaoc.utils.parsing import parse_html_tag
from pyaoc.utils.session import get_aoc_session
from pyaoc.utils.url import get_submit_url

def submit(year: int, day: int, exercise: int, answer: int) -> str:
    url = get_submit_url(year, day)
    session = get_aoc_session()
    req = Request(url,
                  urlencode({'level' : exercise, 'answer' : answer}).encode(),
                  headers={"Cookie": f'session={session}'})
    with urlopen(req) as response:
        response = response.read().decode('utf-8')

    response = parse_html_tag(response, 'article', True)
    return response



