use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use aocsuite_fs::write_with_confirmation;
use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::{
    read_template_contents, result_file, template_filename, AocLanguageResult, Language,
    LanguageFile, LanguageRunner,
};

const LIB_NAME: &str = "aocsuitelib";

pub struct PythonLanguage {
    root_dir: PathBuf,
}
impl PythonLanguage {
    pub fn new(root_dir: PathBuf) -> PythonLanguage {
        PythonLanguage { root_dir }
    }
    fn main_filepath(&self, day: PuzzleDay, year: PuzzleYear) -> PathBuf {
        PathBuf::from(&self.root_dir).join(format!("year{year}_day{day}.py"))
    }
    fn lib_filepath(&self) -> PathBuf {
        PathBuf::from(&self.root_dir).join(PathBuf::from(LIB_NAME).with_extension("py"))
    }
}
impl LanguageRunner for PythonLanguage {
    fn scaffold(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        template_dir: Option<&str>,
        overwrite: bool,
    ) -> AocLanguageResult<()> {
        fs::create_dir_all(&self.root_dir)?;

        if !self.lib_filepath().exists() {
            let lib_contents = default_lib_contents();
            write_with_confirmation(self.lib_filepath(), lib_contents, false)?
        }

        let main_contents = match template_dir {
            Some(dir) => {
                let path = template_filename(dir, &Language::Python);
                read_template_contents(&path)?
            }
            None => default_main_contents(),
        };
        write_with_confirmation(self.main_filepath(day, year), main_contents, overwrite)?;
        Ok(())
    }
    fn compile(&self, _: PuzzleDay, _: PuzzleYear) -> AocLanguageResult<Option<Output>> {
        Ok(None)
    }

    fn run(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        part: &str,
        input: &Path,
    ) -> AocLanguageResult<Output> {
        let entry_point = self.get_path(day, year, LanguageFile::Main);
        let output = Command::new("python3")
            .args([
                entry_point.to_str().unwrap(),
                "-p",
                part,
                "-o",
                result_file().to_str().unwrap(),
                input.to_str().unwrap(),
            ])
            .output()?;

        Ok(output)
    }
    fn get_path(&self, day: PuzzleDay, year: PuzzleYear, file: LanguageFile) -> PathBuf {
        match file {
            LanguageFile::Lib => self.lib_filepath(),
            LanguageFile::Main => self.main_filepath(day, year),
            LanguageFile::Executable => self.get_path(day, year, LanguageFile::Main),
        }
    }
}

pub fn default_lib_contents() -> String {
    let content = format!(
        r#"import argparse
import json
import sys
import time
from typing import Callable


def run_exercises(
    part1: Callable[[str], str],
    part2: Callable[[str], str],
):
    parser = argparse.ArgumentParser(description="Run AoC parts with input and save results.")
    parser.add_argument("input", type=str, help="Path to input file")
    parser.add_argument("--part", "-p", choices=["1", "2", "both"], default="both", help="Which part to run")
    parser.add_argument("--output", "-o", type=str, required=True, help="Path to output result JSON file")

    args = parser.parse_args()

    try:
        with open(args.input, "r") as f:
            input_data = f.read()
    except Exception as e:
        print(f"Failed to read file '{{args.input}}': {{e}}", file=sys.stderr)
        sys.exit(1)

    output = {{
        "part1": None,
        "part2": None,
    }}

    if args.part in ("1", "both"):
        start = time.perf_counter()
        result1 = part1(input_data)
        runtime1 = (time.perf_counter() - start) * 1000
        output["part1"] = {{
            "answer": str(result1),
            "runtime_ms": round(runtime1),
        }}

    if args.part in ("2", "both"):
        start = time.perf_counter()
        result2 = part2(input_data)
        runtime2 = (time.perf_counter() - start) * 1000
        output["part2"] = {{
            "answer": str(result2),
            "runtime_ms": round(runtime2),
        }}

    try:
        with open(args.output, "w") as f:
            json.dump(output, f, indent=2)
    except Exception as e:
        print(f"Failed to write result file '{{args.output}}': {{e}}", file=sys.stderr)
        sys.exit(1)
"#,
    );

    content.to_string()
}
pub fn default_main_contents() -> String {
    let content = format!(
        r#"from {} import run_exercises

"""Implement your solution here"""

def part1(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 1 not implemented yet. Input length: {{len(input)}}"

def part2(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 2 not implemented yet. Input length: {{len(input)}}"


if __name__ == "__main__":
    run_exercises(part1, part2)
"#,
        LIB_NAME
    );

    content.to_string()
}
