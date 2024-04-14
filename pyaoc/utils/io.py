import logging

logger = logging.getLogger()

def save_file(name: str, content: str) -> None:
    open(name, 'w').write(content)
    logger.debug(f'Saved file: {name}')
