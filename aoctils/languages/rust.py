import toml

from aoctils.languages import factory
from aoctils.utils import enums


@factory.register_language(enums.LanguageName.RUST)
class RustAdapter(factory.LanguageAdapter):
    def __init__(
        self, name: enums.LanguageName, base_dir: str, year: int, day: int
    ) -> None:
        super().__init__(name, base_dir, year, day)
        self.template_file = "exercise.rs"
        self.lib_file = "lib.rs"
        self.exercise_bin = f"year{year}_day{day}"
        self.exercise_file = f"{self.exercise_bin}.rs"
        self.cargo_file = "Cargo.toml"

    def apply_base_template(self):
        self.copy_to_directory(
            self.template_directory / self.lib_file,
            self.language_base_dir / "src" / self.lib_file,
        )
        self.copy_to_directory(
            self.template_directory / self.cargo_file,
            self.language_base_dir / self.cargo_file,
        )

    def apply_exercise_template(self):
        self.copy_to_directory(
            self.template_directory / self.template_file,
            self.language_base_dir / "src" / "bin" / self.exercise_file,
        )
        self._register_binary()

    def command(self, exercise, data_path):
        cargo_file = self.language_base_dir / self.cargo_file
        cmd = [
            "cargo",
            "run",
            "--manifest-path",
            str(cargo_file),
            "--bin",
            self.exercise_bin,
            "--",
            "--exercise",
            f"{exercise}",
            "--data-path",
            str(data_path),
        ]
        return cmd

    def get_exercise_path(self) -> str:
        return str(self.language_base_dir / f"src" / "bin" / self.exercise_file)

    def is_initialized(self):
        return (
            (self.language_base_dir / self.cargo_file).exists()
            & (self.language_base_dir / f"src" / self.lib_file).exists()
            & (self.language_base_dir / "src" / "bin" / self.exercise_file).exists()
        )

    def _register_binary(self):
        cargo_toml = toml.load(open(self.language_base_dir / self.cargo_file, "r"))
        if not "bin" in cargo_toml:
            cargo_toml["bin"] = []

        # Check if already registed
        for bin in cargo_toml["bin"]:
            if bin["name"] == self.exercise_bin:
                return

        cargo_toml["bin"].append(
            {
                "name": self.exercise_bin,
                "path": "/".join(
                    str(
                        self.language_base_dir / "src" / "bin" / self.exercise_file
                    ).split("/")[1:]
                ),
            }
        )
        toml.dump(cargo_toml, open(self.language_base_dir / self.cargo_file, "w"))
        return
