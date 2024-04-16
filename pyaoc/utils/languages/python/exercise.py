
def create_python_exercise()-> str:
    return """from utils import run_python_exercise # type: ignore

def exercise(data: str) -> int:

    return 0

if __name__ == '__main__':
    run_python_exercise(exercise)  
"""
