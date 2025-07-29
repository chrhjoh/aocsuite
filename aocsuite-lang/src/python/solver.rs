use crate::{
    traits::Solver,
    utils::{AocLanguageResult, SolveFile},
};

use super::PythonRunner;

impl Solver for PythonRunner {
    fn compile(
        &self,
        _day: aocsuite_utils::PuzzleDay,
        _year: aocsuite_utils::PuzzleYear,
    ) -> AocLanguageResult<Option<std::process::Output>> {
        Ok(None)
    }

    fn run(
        &self,
        _day: aocsuite_utils::PuzzleDay,
        _year: aocsuite_utils::PuzzleYear,
        part: &str,
        input: &std::path::Path,
        output: &std::path::Path,
    ) -> AocLanguageResult<std::process::Output> {
        let python_path = self.get_python_path();

        let output = std::process::Command::new(python_path)
            .arg(self.get_solvefile_path(&SolveFile::Main))
            .arg(input)
            .arg(output)
            .arg(part)
            .current_dir(&self.root_dir)
            .output()?;

        Ok(output)
    }

    fn setup_solver(&self) -> AocLanguageResult<()> {
        Ok(())
    }

    fn get_solvefile_path(&self, file: &SolveFile) -> std::path::PathBuf {
        match file {
            SolveFile::Main => self.root_dir.join("main.py"),
            SolveFile::TemplateSolution => self.root_dir.join("template.py"),
            SolveFile::LinkedSolution(_) => self.root_dir.join("solution.py"),
            SolveFile::Solution(day, year) => self
                .root_dir
                .join(format!("year{year}"))
                .join(format!("day{day}.py")),
        }
    }
    fn template_contents(&self) -> String {
        r#""""Implement your solution here"""

def part1(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 1 not implemented yet. Input length: {{len(input)}}"

def part2(input: str) -> str:
    # Replace this stub with actual implementation
    return f"Part 2 not implemented yet. Input length: {{len(input)}}"

"#
        .to_string()
    }
    fn main_contents(&self) -> String {
        r#"import sys
import json
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
    
    # Write output to file
    try:
        with open(output_file, 'w') as f:
            json.dump(output, f, indent=2)
    except IOError as e:
        print(f"Failed to write output file '{output_file}': {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
"#
        .to_string()
    }
}
