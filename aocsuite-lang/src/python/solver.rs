use std::process::Output;

use crate::{
    traits::Solver,
    utils::{AocLanguageResult, SolverFile},
    AocLanguageError,
};
use aocsuite_utils::{execute_command, CommandRequest, PuzzleId};

use super::PythonRunner;

impl Solver for PythonRunner<'_> {
    fn compile(&self) -> AocLanguageResult<Option<Output>> {
        Ok(None)
    }

    fn run(
        &self,
        part: aocsuite_utils::PartSelection,
        input: &std::path::Path,
        output: &std::path::Path,
    ) -> AocLanguageResult<Output> {
        let python_path = self.get_python_path();

        Ok(execute_command(
            self.executor,
            CommandRequest::new(python_path)
                .arg(self.solver_file_path(&SolverFile::Entrypoint))
                .arg(input)
                .arg(output)
                .arg(part.to_string())
                .current_dir(&self.root_dir),
        )?)
    }
    fn clean_cache(&self) -> AocLanguageResult<()> {
        Err(AocLanguageError::Clean(
            "Python language have no files to clean.".to_string(),
        ))
    }

    fn migrate_runtime(&self) -> AocLanguageResult<()> {
        crate::runtime::migrate_runtime(
            &self.root_dir,
            vec![(
                self.solver_file_path(&SolverFile::Entrypoint),
                self.main_contents(),
            )],
        )
    }

    fn solver_file_path(&self, file: &SolverFile) -> std::path::PathBuf {
        match file {
            SolverFile::Entrypoint => self.root_dir.join("main.py"),
            SolverFile::SolutionTemplate => self.root_dir.join("template.py"),
            SolverFile::ActiveSolution(_, _) => self.root_dir.join("solution.py"),
            SolverFile::PuzzleSolution(day, year) => self
                .root_dir
                .join("solutions")
                .join(format!("{}.py", PuzzleId::new(*day, *year))),
        }
    }
    fn template_contents(&self) -> String {
        r#""""Implement your solution here"""

def part1(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 1 not implemented yet. Input length: {len(input)}"

def part2(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 2 not implemented yet. Input length: {len(input)}"

"#
        .to_string()
    }
    fn main_contents(&self) -> String {
        r#"import sys
import json
import os
import time
from pathlib import Path

# Import solution functions
from solution import part1, part2

def run_part(part_fn, input_data):
    """Run a part function and measure execution time."""
    start = time.perf_counter()
    answer = part_fn(input_data)
    runtime_ms = int((time.perf_counter() - start) * 1000)
    
    return {
        "answer": str(answer),
        "runtime_ms": runtime_ms
    }

def main():
    if len(sys.argv) < 3 or len(sys.argv) > 4:
        print(f"Usage: {sys.argv[0]} <input_file> <output_file> [1|2|both]", file=sys.stderr)
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    part = sys.argv[3] if len(sys.argv) > 3 else "both"
    
    # Validate part argument
    if part not in ["1", "2", "both"]:
        print(f"Invalid part '{part}'. Use '1', '2', or 'both'", file=sys.stderr)
        sys.exit(1)
    
    # Read input file
    try:
        with open(input_file, 'r') as f:
            input_data = f.read()
    except IOError as e:
        print(f"Failed to read file '{input_file}': {e}", file=sys.stderr)
        sys.exit(1)
    
    # Ensure output directory exists
    output_path = Path(output_file)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Prepare output structure
    output = {
        "part1": None,
        "part2": None
    }
    
    # Run requested parts
    if part == "1":
        output["part1"] = run_part(part1, input_data)
    elif part == "2":
        output["part2"] = run_part(part2, input_data)
    else:  # both
        output["part1"] = run_part(part1, input_data)
        output["part2"] = run_part(part2, input_data)
    
    # Publish a complete result so readers never observe a partial JSON document.
    try:
        temporary_output_path = output_path.with_suffix('.tmp')
        with open(temporary_output_path, 'w') as f:
            json.dump(output, f, indent=2)
        os.replace(temporary_output_path, output_path)
    except IOError as e:
        print(f"Failed to write output file '{output_file}': {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
"#
        .to_string()
    }
}
