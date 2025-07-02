use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::{fs, io};

use toml_edit::Array;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Value;
use toml_edit::table;
use toml_edit::value;

use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::{AocLanguageResult, LanguageFile};

use super::LanguageRunner;

pub struct RustLanguage {
    root_dir: PathBuf,
}

const CARGO_FILE: &str = "Cargo.toml";
const LIB_FILE: &str = "lib.rs";
const MAIN_FILE: &str = "main.rs";

impl RustLanguage {
    pub fn new(root_dir: PathBuf) -> RustLanguage {
        RustLanguage { root_dir }
    }
    fn package_name(&self, day: PuzzleDay, year: PuzzleYear) -> String {
        format!("year{year}_day{day}").to_owned()
    }
    fn package_dir(&self, day: PuzzleDay, year: PuzzleYear) -> PathBuf {
        PathBuf::from(format!("year{year}")).join(format!("day{day}"))
    }
}

impl LanguageRunner for RustLanguage {
    fn scaffold(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        template_dir: Option<&str>,
    ) -> AocLanguageResult<()> {
        fs::create_dir_all(&self.root_dir)?;
        let root_cargo_path = self.root_dir.join(CARGO_FILE);
        let package_dir = self.package_dir(day, year);
        update_root_cargo(&root_cargo_path, &package_dir)?;
        let package_name = self.package_name(day, year);

        let package_path = self.root_dir.join(&package_dir);

        match template_dir {
            Some(_) => unimplemented!(),
            None => create_default_exercise_package(&package_path, &package_name)?,
        }
        Ok(())
    }
    fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> AocLanguageResult<Option<Output>> {
        let package_name = self.package_name(day, year);
        let output = Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg(package_name)
            .current_dir(&self.root_dir)
            .output()?;
        Ok(Some(output))
    }

    fn run(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        part: &str,
        input: &Path,
    ) -> AocLanguageResult<Output> {
        let package_name = self.package_name(day, year);

        let binary_path = self
            .root_dir
            .join("target")
            .join("debug")
            .join(package_name);

        let output = Command::new(binary_path)
            .args([input.to_str().unwrap(), part])
            .output()?;

        Ok(output)
    }
    fn get_path(&self, day: PuzzleDay, year: PuzzleYear, file: crate::LanguageFile) -> PathBuf {
        match file {
            LanguageFile::Lib => self
                .root_dir
                .join(self.package_dir(day, year))
                .join("src")
                .join("lib.rs"),
            LanguageFile::Main => self
                .root_dir
                .join(self.package_dir(day, year))
                .join("src")
                .join("main.rs"),
        }
    }
}

fn update_root_cargo(root_cargo_path: &Path, package_path: &Path) -> AocLanguageResult<()> {
    // check if cargo.toml exists. if not create it, then add the member to it.
    if !root_cargo_path.exists() {
        write_root_cargo(&root_cargo_path)?;
    }
    // read the root cargo file and add member to it.
    let contents = fs::read_to_string(&root_cargo_path)?;
    let mut doc = contents.parse::<DocumentMut>()?;
    let workspace = doc["workspace"].or_insert(table());
    let package_path = package_path
        .to_str()
        .expect("Package should should be UTF-8");

    match workspace["members"].as_array_mut() {
        None => {
            let mut array = Array::new();
            array.push(package_path);
            workspace["members"] = Item::Value(Value::Array(array));
        }
        Some(array) => {
            if !array.iter().any(|v| v.as_str() == Some(package_path)) {
                array.push(package_path);
            }
        }
    }
    fs::write(root_cargo_path, doc.to_string())?;
    Ok(())
}

fn create_default_exercise_package(package_path: &Path, package_name: &str) -> io::Result<()> {
    fs::create_dir_all(&package_path.join("src"))?;
    write_default_exercise_cargo(&package_path.join(CARGO_FILE), package_name)?;
    write_default_main_file(&package_path.join("src").join(MAIN_FILE), package_name)?;
    write_default_lib_file(&package_path.join("src").join(LIB_FILE))?;

    Ok(())
}

fn write_root_cargo(cargo_toml_path: &Path) -> io::Result<()> {
    let mut doc = DocumentMut::new();
    doc["workspace"] = table();
    doc["workspace"]["resolver"] = value("3");

    std::fs::write(cargo_toml_path, doc.to_string())
}

// only init if template doesnt exist.
fn write_default_exercise_cargo(cargo_toml_path: &Path, package_name: &str) -> io::Result<()> {
    let mut doc = DocumentMut::new();
    doc["package"] = table();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2024");

    std::fs::write(cargo_toml_path, doc.to_string())?;
    Ok(())
}

pub fn write_default_main_file(main_file_path: &Path, package_name: &str) -> io::Result<()> {
    let content = format!(
        r#"use std::{{env, fs, process}};
use {package}::{{part1, part2}};

fn print_usage(program_name: &str) {{
    eprintln!("Usage: {{}} <input_file> [1|2|both]", program_name);
    eprintln!("If no part is specified, runs both parts.");
}}

fn main() {{
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {{
        print_usage(&args[0]);
        process::exit(1);
    }}

    let input_path = &args[1];
    let part_to_run = if args.len() == 3 {{
        args[2].to_lowercase()
    }} else {{
        "both".to_string()
    }};

    let input = match fs::read_to_string(input_path) {{
        Ok(content) => content,
        Err(e) => {{
            eprintln!("Failed to read file '{{}}': {{}}", input_path, e);
            process::exit(1);
        }}
    }};

    match part_to_run.as_str() {{
        "1" => {{
            let result = part1(&input);
            println!("Result Part 1:\n{{}}", result);
        }}
        "2" => {{
            let result = part2(&input);
            println!("Result Part 2:\n{{}}", result);
        }}
        "both" => {{
            let result1 = part1(&input);
            println!("Result Part 1:\n{{}}", result1);

            let result2 = part2(&input);
            println!("Result Part 2:\n{{}}", result2);
        }}
        _ => {{
            eprintln!("Invalid part argument: '{{}}'", part_to_run);
            print_usage(&args[0]);
            process::exit(1);
        }}
    }}
}}
"#,
        package = package_name
    );

    std::fs::write(main_file_path, content)
}

pub fn write_default_lib_file(lib_file_path: &Path) -> io::Result<()> {
    let content = r#"/// Implement your solution here

/// Solve part 1 of the puzzle
pub fn part1(input: &str) -> String {
    // Replace this stub with actual implementation
    format!("Part 1 not implemented yet. Input length: {}", input.len())
}

/// Solve part 2 of the puzzle
pub fn part2(input: &str) -> String {
    // Replace this stub with actual implementation
    format!("Part 2 not implemented yet. Input length: {}", input.len())
}
"#;
    std::fs::write(lib_file_path, content)
}
