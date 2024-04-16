
def create_python_exercise()-> str:
    return """from .utils import run_python_exercise

def exercise(data: str) -> int:

    return 0

if __name__ == '__main__':
    run_python_exercise(exercise)    

    """
