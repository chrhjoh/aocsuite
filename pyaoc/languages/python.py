import os
import shutil

from pyaoc.languages import factory
from pyaoc.utils import enums


@factory.register_language(enums.LanguageName.PYTHON)
class PythonAdapter(factory.LanguageAdapter):
    def __init__(self, name: enums.LanguageName, directory: str) -> None:
        super().__init__(name, directory)
        self.exercise_file = "exercise.py"
        self.main_file = "main.py"

    def initialize(self):
        shutil.copy(
            os.path.join(self.template_directory, self.exercise_file),
            os.path.join(self.directory, self.exercise_file),
        )
        shutil.copy(
            os.path.join(self.template_directory, self.exercise_file),
            os.path.join(self.directory, self.main_file),
        )

    def command(self, exercise, data_path, answer_path):
        main_path = os.path.join(self.directory, self.main_file)
        cmd = [
            "python3",
            main_path,
            "--exercise",
            f"{exercise}",
            "--data-path",
            data_path,
        ]
        if answer_path:
            cmd.extend(["--answer-path", answer_path])
        return cmd
