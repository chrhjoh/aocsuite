import shutil

from pyaoc.aoc_directory import AocDirectory
from pyaoc.languages import factory
from pyaoc.utils import enums


@factory.register_language(enums.LanguageName.PYTHON)
class PythonAdapter(factory.LanguageAdapter):
    def __init__(self, name: enums.LanguageName, directory: AocDirectory) -> None:
        super().__init__(name, directory)
        self.exercise_file = "exercise.py"
        self.main_file = "main.py"

    def fetch(self):
        self.working_directory.copy_files(
            [
                self.template_directory / self.exercise_file,
                self.template_directory / self.main_file,
            ]
        )

    def command(self, exercise, data_path):
        main_path = self.working_directory / self.main_file
        cmd = [
            "python3",
            main_path,
            "--exercise",
            f"{exercise}",
            "--data-path",
            data_path,
        ]
        return cmd

    def get_exercise_name(self):
        return self.exercise_file
