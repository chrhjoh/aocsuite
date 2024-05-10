import logging
import os
import shutil
import subprocess
from pathlib import Path
from typing import Optional

from pyaoc.utils import enums
from pyaoc.utils.languages import factory

logger = logging.getLogger(__file__)


@factory.register_language(enums.LanguageName.PYTHON)
class PythonAdapter(factory.LanguageAdapter):
    def __init__(self, directory: str) -> None:
        super().__init__(directory)
        template_dir = Path(__file__).parent / "templates"
        self.exercise_template_path = str((template_dir / "exercise.py").resolve())
        self.utils_template_path = str((template_dir / "utils.py").resolve())
        self.base_exercise_path = str(Path(directory) / "exercise{exercise}.py")
        self.utils_path = str(Path(directory) / "utils.py")

    def execute(self, exercise: int, data_path: str, answer_path: Optional[str]) -> None:

        exercise_path = self.base_exercise_path.format(exercise=exercise)
        cmd = ["python3", exercise_path, "--data-path", data_path]
        if answer_path:
            cmd.extend(["--answer-path", answer_path])
        try:
            subprocess.run(cmd, check=True)
        except subprocess.CalledProcessError as err:
            logger.error(
                f"Error was encountered while running executor. Error Encountered:\n{err}"
            )

    def initialize(self):
        for exercise in range(1, 3):
            shutil.copy(
                self.exercise_template_path, self.base_exercise_path.format(exercise=exercise)
            )
        shutil.copy(self.utils_template_path, self.utils_path)
