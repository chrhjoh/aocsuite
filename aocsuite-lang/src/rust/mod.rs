use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use aocsuite_fs::write_with_confirmation;
use toml_edit::table;
use toml_edit::value;
use toml_edit::Array;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Value;

use aocsuite_utils::{PuzzleDay, PuzzleYear};

use crate::{read_template_contents, result_filename, AocLanguageResult, LanguageFile};

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
        overwrite: bool,
    ) -> AocLanguageResult<()> {
        fs::create_dir_all(&self.root_dir)?;
        let root_cargo_path = self.root_dir.join(CARGO_FILE);
        let package_dir = self.package_dir(day, year);
        update_root_cargo(&root_cargo_path, &package_dir)?;
        let package_name = self.package_name(day, year);

        let package_path = self.root_dir.join(&package_dir);
        let result_file = result_filename(day, year);

        create_exercise_package(
            template_dir,
            &package_path,
            &package_name,
            &result_file,
            overwrite,
        )?;
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
        let binary = self.get_path(day, year, LanguageFile::Executable);

        let output = Command::new(binary)
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
                .join(LIB_FILE),
            LanguageFile::Main => self
                .root_dir
                .join(self.package_dir(day, year))
                .join("src")
                .join(MAIN_FILE),
            LanguageFile::Executable => self
                .root_dir
                .join("target")
                .join("debug")
                .join(self.package_name(day, year)),
        }
    }
}

fn update_root_cargo(root_cargo_path: &Path, package_path: &Path) -> AocLanguageResult<()> {
    // check if cargo.toml exists. if not create it, then add the member to it.
    if !root_cargo_path.exists() {
        let contents = root_cargo_contents();
        write_with_confirmation(root_cargo_path, contents, true)?
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
    write_with_confirmation(root_cargo_path, doc.to_string(), true)?;
    Ok(())
}

fn create_exercise_package(
    template_dir: Option<&str>,
    package_path: &Path,
    package_name: &str,
    result_file: &str,
    overwrite: bool,
) -> AocLanguageResult<()> {
    fs::create_dir_all(&package_path.join("src"))?;
    let cargo_contents = default_exercise_cargo_contents(&package_name);
    let main_contents = default_main_contents(&package_name, result_file);
    let lib_contents = match template_dir {
        Some(dir) => {
            let path = Path::new(&dir).join("rust").join(LIB_FILE);
            read_template_contents(&path)?
        }
        None => default_lib_contents(),
    };
    let contents = vec![cargo_contents, main_contents, lib_contents];
    let file_paths = vec![
        package_path.join(CARGO_FILE),
        package_path.join("src").join(MAIN_FILE),
        package_path.join("src").join(LIB_FILE),
    ];
    for (path, content) in file_paths.iter().zip(contents) {
        write_with_confirmation(path, content, overwrite)?;
    }

    Ok(())
}

fn root_cargo_contents() -> String {
    let mut doc = DocumentMut::new();
    doc["workspace"] = table();
    doc["workspace"]["resolver"] = value("2");
    let deps = doc["workspace"]["dependencies"].or_insert(toml_edit::table());

    let mut serde_table = toml_edit::table();
    serde_table["version"] = value("1.0");

    let mut features = Array::default();
    features.push("derive");
    serde_table["features"] = Item::Value(features.into());

    deps["serde"] = serde_table;

    // Add serde_json with just version string
    deps["serde_json"] = value("1.0");

    doc.to_string()
}

// only init if template doesnt exist.
fn default_exercise_cargo_contents(package_name: &str) -> String {
    let mut doc = DocumentMut::new();
    doc["package"] = table();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    let deps = doc["dependencies"].or_insert(toml_edit::table());

    let mut serde = toml_edit::table();
    serde["workspace"] = value(true);
    deps["serde"] = serde;

    let mut serde_json = toml_edit::table();
    serde_json["workspace"] = value(true);
    deps["serde_json"] = serde_json;
    doc.to_string()
}

pub fn default_main_contents(package_name: &str, output_file: &str) -> String {
    let content = format!(
        r#"use serde::Serialize;
use std::time::Instant;
use std::{{env, fs, process}};
use {package_name}::{{part1, part2}};

fn print_usage(program_name: &str) {{
    eprintln!("Usage: {{}} <input_file> [1|2|both]", program_name);
    eprintln!("If no part is specified, runs both parts.");
}}

#[derive(Serialize)]
struct PartResult {{
    answer: String,
    runtime_ms: u128,
}}

#[derive(Serialize)]
struct OutputJson {{
    part1: Option<PartResult>,
    part2: Option<PartResult>,
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

    let mut output = OutputJson {{
        part1: None,
        part2: None,
    }};

    if part_to_run == "1" || part_to_run == "both" {{
        let start = Instant::now();
        let answer = part1(&input);
        let runtime = start.elapsed().as_millis();
        output.part1 = Some(PartResult {{
            answer,
            runtime_ms: runtime,
        }});
    }}

    if part_to_run == "2" || part_to_run == "both" {{
        let start = Instant::now();
        let answer = part2(&input);
        let runtime = start.elapsed().as_millis();
        output.part2 = Some(PartResult {{
            answer,
            runtime_ms: runtime,
        }});
    }}

    let json_string = serde_json::to_string_pretty(&output).expect("Failed to serialize JSON");
    let output_file = "{output_file}";
    if let Err(e) = fs::write(output_file, json_string) {{
        eprintln!("Failed to write JSON output file '{{}}': {{}}",output_file,e);
        process::exit(1);
    }}
}}
"#,
    );

    content
}

pub fn default_lib_contents() -> String {
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
    content.to_string()
}
