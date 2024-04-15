from pyaoc.utils.http import initialize_http
from pyaoc.utils.parsing import parse_html_tag
from pyaoc.utils import messages
import logging

logger = logging.getLogger(__file__)

def submit(year: int, day: int, exercise: int, answer: int) -> str:
    http = initialize_http(year, day)
    response = http.post_answer(answer, exercise)
    response = parse_html_tag(response, 'article', True)
    response = response[:response.find('[')]

    logger.debug(messages.DEBUG_PARSED_SUBMIT_RESPONSE.format(response=response))

    return response


