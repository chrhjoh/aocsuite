# cargo add clap --features derive
from aocli.aoc_directory import AocDirectory
from aocli.languages import factory
from aocli.utils import enums


@factory.register_language(enums.LanguageName.RUST)
class RustAdapter(factory.LanguageAdapter):
    def __init__(
        self, name: enums.LanguageName, working_directory: AocDirectory
    ) -> None:
        super().__init__(name, working_directory)
        self.exercise_file = "exercise.rs"
        self.main_file = "main.rs"
        self.cargo_file = "Cargo.toml"

    def fetch(self):
        self.working_directory.copy_files(
            [
                self.template_directory / self.exercise_file,
                self.template_directory / self.main_file,
                self.template_directory / self.cargo_file,
            ]
        )

    def command(self, exercise, data_path):
        cargo_file = self.working_directory / self.cargo_file
        cmd = [
            "cargo",
            "run",
            "--manifest-path",
            str(cargo_file),
            "--",
            "--exercise",
            f"{exercise}",
            "--data-path",
            str(data_path),
        ]
        return cmd

    def get_exercise_name(self):
        return self.exercise_file
