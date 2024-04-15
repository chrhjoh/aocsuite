from pyaoc.utils.http import initialize_http
from pyaoc.utils.parsing import parse_calendar

def get_calendar(year: int):
    request = initialize_http(year=year)
    raw_calendar = request.get_calendar()
    return raw_calendar


def show_calendar(calendar):
    print(calendar)

def calendar(year: int):
    calendar = get_calendar(year)
    calendar = parse_calendar(calendar)
    calendar = pad_calendar(calendar)
    show_calendar(calendar) 

def pad_calendar(calendar: str):
    list_calendar = calendar.split('\n')
    padded_calendar = []
    max_length = max([len(line) + 1 for line in list_calendar])
    max_stars = max([line.strip().count('*', len(line.strip())-3, len(line.strip())) for line in list_calendar])
    for line in calendar.split('\n'):
        length_diff = max_length - len(line)
        stars_diff = max_stars - line.strip().count('*', len(line.strip())-3, len(line.strip()))
        line = ' ' * (length_diff - stars_diff + 1) + line 
        padded_calendar.append(line)
    return '\n'.join(padded_calendar)

