use crate::traits::Solver;
use crate::utils::{AocLanguageResult, SolveFile};

use super::RustRunner;

impl Solver for RustRunner {
    fn compile(
        &self,
        _day: aocsuite_utils::PuzzleDay,
        _year: aocsuite_utils::PuzzleYear,
    ) -> AocLanguageResult<Option<std::process::Output>> {
        let output = std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&self.root_dir)
            .output()?;
        Ok(Some(output))
    }

    fn run(
        &self,
        _day: aocsuite_utils::PuzzleDay,
        _year: aocsuite_utils::PuzzleYear,
        part: &str,
        input: &std::path::Path,
        output: &std::path::Path,
    ) -> AocLanguageResult<std::process::Output> {
        let binary_path = self
            .root_dir
            .join("target")
            .join("release")
            .join("aocsuite_solution_rust");

        let output = std::process::Command::new(&binary_path)
            .arg(input)
            .arg(output)
            .arg(part)
            .current_dir(&self.root_dir)
            .output()?;

        Ok(output)
    }

    fn setup_solver(&self) -> AocLanguageResult<()> {
        // Need main always for editing files
        let main_path = self.get_solvefile_path(&SolveFile::Main);
        if !main_path.exists() {
            std::fs::write(&main_path, self.main_contents())?
        }
        Ok(())
    }
    fn clean_cache(&self) -> AocLanguageResult<()> {
        std::process::Command::new("cargo")
            .arg("clean")
            .current_dir(&self.root_dir)
            .status()?;

        Ok(())
    }

    fn get_solvefile_path(&self, file: &SolveFile) -> std::path::PathBuf {
        match file {
            SolveFile::Main => self.src_dir().join("main.rs"),
            SolveFile::TemplateSolution => self.root_dir.join("template.rs"),
            SolveFile::LinkedSolution(_) => self.src_dir().join("solution.rs"),
            SolveFile::Solution(day, year) => self
                .root_dir
                .join(format!("year{year}"))
                .join(format!("day{day}.rs")),
        }
    }
    fn template_contents(&self) -> String {
        "/// Implement your solution here

/// Solve part 1 of the puzzle
pub fn part1(input: &str) -> String {
    unimplemented!()
}

/// Solve part 2 of the puzzle
pub fn part2(input: &str) -> String {
    unimplemented!()
}"
        .to_string()
    }
    fn main_contents(&self) -> String {
        r#"use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path, process, time::Instant};
mod solution;
use solution::{part1, part2};

#[derive(Debug)]
enum Part {
    One,
    Two,
    Both,
}

#[derive(Serialize, Deserialize)]
struct PartResult {
    answer: String,
    runtime_ms: u128,
}

#[derive(Serialize, Deserialize)]
struct OutputJson {
    part1: Option<PartResult>,
    part2: Option<PartResult>,
}

fn run_part<F>(part_fn: F, input: &str) -> PartResult
where
    F: FnOnce(&str) -> String,
{
    let start = Instant::now();
    let answer = part_fn(input);
    let runtime_ms = start.elapsed().as_millis();

    PartResult { answer, runtime_ms }
}

fn run<F1, F2>(part1_fn: F1, part2_fn: F2)
where
    F1: FnOnce(&str) -> String,
    F2: FnOnce(&str) -> String,
{
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 4 {
        eprintln!("Usage: {} <input_file> <output_file> [1|2|both]", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];
    let part = match args.get(3).map(|s| s.as_str()) {
        Some("1") => Part::One,
        Some("2") => Part::Two,
        Some("both") | None => Part::Both,
        Some(other) => {
            eprintln!("Invalid part '{}'. Use '1', '2', or 'both'", other);
            process::exit(1);
        }
    };

    // Read input file
    let input = fs::read_to_string(input_file).unwrap_or_else(|e| {
        eprintln!("Failed to read file '{}': {}", input_file, e);
        process::exit(1);
    });

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_file).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create output directory: {}", e);
            process::exit(1);
        }
    }

    let mut output = OutputJson {
        part1: None,
        part2: None,
    };

    match part {
        Part::One => output.part1 = Some(run_part(part1_fn, &input)),
        Part::Two => output.part2 = Some(run_part(part2_fn, &input)),
        Part::Both => {
            output.part1 = Some(run_part(part1_fn, &input));
            output.part2 = Some(run_part(part2_fn, &input));
        }
    }

    // Write output to file
    let json_string = serde_json::to_string_pretty(&output).expect("Failed to serialize JSON");
    if let Err(e) = fs::write(output_file, json_string) {
        eprintln!("Failed to write output file '{}': {}", output_file, e);
        process::exit(1);
    }
}

fn main() {
    run(part1, part2);
}

"#
        .to_string()
    }
}
