use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use aocsuite_fs::write_with_confirmation;
use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::{
    read_template_contents, result_filename, AocLanguageResult, LanguageFile, LanguageRunner,
};

const MAIN_FILE: &str = "main.py";
const LIB_FILE: &str = "lib.py";

pub struct PythonLanguage {
    root_dir: PathBuf,
}
impl PythonLanguage {
    pub fn new(root_dir: PathBuf) -> PythonLanguage {
        PythonLanguage { root_dir }
    }
    fn package_dir(&self, day: PuzzleDay, year: PuzzleYear) -> PathBuf {
        PathBuf::from(format!("year{year}")).join(format!("day{day}"))
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
        let package_dir = self.package_dir(day, year);
        let package_path = self.root_dir.join(&package_dir);
        fs::create_dir_all(&package_path)?;
        let result_file = result_filename(day, year);

        create_exercise_package(template_dir, &package_path, &result_file, overwrite)?;
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
            .args([entry_point.to_str().unwrap(), input.to_str().unwrap(), part])
            .output()?;

        Ok(output)
    }
    fn get_path(&self, day: PuzzleDay, year: PuzzleYear, file: LanguageFile) -> PathBuf {
        match file {
            LanguageFile::Lib => self
                .root_dir
                .join(self.package_dir(day, year))
                .join(LIB_FILE),
            LanguageFile::Main => self
                .root_dir
                .join(self.package_dir(day, year))
                .join(MAIN_FILE),
            LanguageFile::Executable => self.get_path(day, year, LanguageFile::Main),
        }
    }
}
fn create_exercise_package(
    template_dir: Option<&str>,
    package_path: &Path,
    result_file: &str,
    overwrite: bool,
) -> AocLanguageResult<()> {
    let main_contents = default_main_contents(result_file);
    let lib_contents = match template_dir {
        Some(dir) => {
            let path = Path::new(&dir).join("python").join(LIB_FILE);
            read_template_contents(&path)?
        }
        None => default_lib_contents(),
    };
    let contents = vec![main_contents, lib_contents];
    let file_paths = vec![package_path.join(MAIN_FILE), package_path.join(LIB_FILE)];
    for (path, content) in file_paths.iter().zip(contents) {
        write_with_confirmation(path, content, overwrite)?;
    }

    pub fn default_main_contents(output_file: &str) -> String {
        let content = format!(
            r#"import sys
import time
import json

from lib import part1, part2

def main():
    args = sys.argv

    if len(args) < 2 or len(args) > 3:
        print_usage(args[0])
        sys.exit(1)

    input_path = args[1]
    part_to_run = args[2].lower() if len(args) == 3 else "both"

    try:
        with open(input_path, "r") as f:
            input_data = f.read()
    except Exception as e:
        print(f"Failed to read file '{{input_path}}': {{e}}", file=sys.stderr)
        sys.exit(1)

    output = {{
        "part1": None,
        "part2": None,
    }}

    if part_to_run in ("1", "both"):
        start = time.perf_counter()
        result1 = part1(input_data)
        runtime1 = (time.perf_counter() - start) * 1000  # ms
        output["part1"] = {{
            "answer": str(result1),
            "runtime_ms": round(runtime1),
        }}

    if part_to_run in ("2", "both"):
        start = time.perf_counter()
        result2 = part2(input_data)
        runtime2 = (time.perf_counter() - start) * 1000  # ms
        output["part2"] = {{
            "answer": str(result2),
            "runtime_ms": round(runtime2),
        }}

    output_file = "{output_file}"

    try:
        with open(output_file, "w") as f:
            json.dump(output, f, indent=2)
    except Exception as e:
        print(f"Failed to write result file '{{output_file}}': {{e}}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
"#,
        );

        content.to_string()
    }
    pub fn default_lib_contents() -> String {
        let content = r#""""Implement your solution here"""

def part1(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 1 not implemented yet. Input length: {len(input)}"

def part2(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 2 not implemented yet. Input length: {len(input)}"
"#;

        content.to_string()
    }

    Ok(())
}
