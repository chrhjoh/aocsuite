from aoctils.languages import factory
from aoctils.utils import enums


@factory.register_language(enums.LanguageName.PYTHON)
class PythonAdapter(factory.LanguageAdapter):
    def __init__(
        self, name: enums.LanguageName, base_dir: str, day: int, year: int
    ) -> None:
        super().__init__(name, base_dir, year, day)
        self.template_file = "exercise.py"
        self.exercise_file = "day{day}.py"
        self.utils_file = "utils.py"
        self.day = day
        self.year = year

    def apply_base_template(self):
        self.copy_to_directory(
            self.template_directory / self.utils_file,
            self.language_base_dir / self.utils_file,
        )

    def apply_exercise_template(self):
        self.copy_to_directory(
            self.template_directory / "exercise.py",
            self.language_base_dir
            / f"year{self.year}"
            / self.exercise_file.format(day=self.day),
        )

    def command(self, exercise, data_path):
        exercise_path = self.get_exercise_path().split(".")[0]
        cmd = [
            "python3",
            "-m",
            exercise_path.replace("/", "."),
            "--exercise",
            f"{exercise}",
            "--data-path",
            data_path,
        ]
        return cmd

    def get_exercise_path(self) -> str:
        return str(
            self.language_base_dir
            / f"year{self.year}"
            / self.exercise_file.format(day=self.day)
        )

    def is_initialized(self):
        return (self.language_base_dir / self.utils_file).exists() & (
            self.language_base_dir
            / f"year{self.year}"
            / self.exercise_file.format(day=self.day)
        ).exists()
