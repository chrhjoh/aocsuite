//TODO: Inspiration from https://github.com/coriolinus/adventofcode-2020/blob/master/day-template/src/main.rs
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use toml_edit::Array;
use toml_edit::DocumentMut;
use toml_edit::Item;
use toml_edit::Value;
use toml_edit::table;
use toml_edit::value;

use crate::AocError;
use crate::AocResult;
use crate::error::AocExecutionError;
use crate::error::AocIoError;
use crate::error::AocParseError;
use crate::utils::copy_file_from_template;
use crate::utils::create_dirs;
use crate::utils::write_file;
use crate::utils::{PuzzleDay, PuzzleYear};

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
}

impl LanguageRunner for RustLanguage {
    fn scaffold(
        &self,
        day: PuzzleDay,
        year: PuzzleYear,
        template_dir: Option<String>,
    ) -> AocResult<()> {
        create_dirs(&self.root_dir)?;
        let root_cargo_path = self.root_dir.join(CARGO_FILE);
        update_root_cargo(&root_cargo_path, day, year)?;
        let package_name = package_path_from_root(day, year);
        let package_path = self.root_dir.join(&package_name);

        create_exercise_package(
            &package_path,
            &package_name
                .to_str()
                .expect("Path name should be valid UTF-8"),
            template_dir,
        )?;
        Ok(())
    }
    fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> AocResult<Option<Output>> {
        let package_name = package_path_from_root(day, year);
        let output = Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg(package_name)
            .current_dir(&self.root_dir)
            .output()
            .map_err(AocExecutionError::Compile)?;
        Ok(Some(output))
    }
    fn run(&self, day: PuzzleDay, year: PuzzleYear) -> AocResult<Option<Output>> {
        let package_name = package_path_from_root(day, year);
        let binary_path = self
            .root_dir
            .join("target")
            .join("debug")
            .join(package_name);

        let output = Command::new(binary_path)
            .current_dir(&self.root_dir)
            .output()
            .map_err(AocExecutionError::Run)?;
        Ok(Some(output))
    }
}

fn update_root_cargo(root_cargo_path: &Path, day: PuzzleDay, year: PuzzleYear) -> AocResult<()> {
    // check if cargo.toml exists. if not create it, then add the member to it.
    if !root_cargo_path.exists() {
        write_root_cargo(&root_cargo_path)?;
    }
    // read the root cargo file and add member to it.
    let contents = fs::read_to_string(&root_cargo_path).map_err(AocIoError::WriteError)?;
    let mut doc = contents
        .parse::<DocumentMut>()
        .map_err(AocParseError::TomlParseError)?;
    let workspace = doc["workspace"].or_insert(table());

    let package_name = package_path_from_root(day, year);
    let package_name = package_name.to_str().expect("Path should be valid UTF-8");

    match workspace["members"].as_array_mut() {
        None => {
            let mut array = Array::new();
            array.push(package_name);
            workspace["members"] = Item::Value(Value::Array(array));
        }
        Some(array) => {
            if !array.iter().any(|v| v.as_str() == Some(package_name)) {
                array.push(package_name);
            }
        }
    }
    fs::write(root_cargo_path, doc.to_string()).map_err(|e| AocError::Io(AocIoError::WriteError(e)))
}

fn create_exercise_package(
    package_path: &Path,
    package_name: &str,
    template_dir: Option<String>,
) -> AocResult<()> {
    fs::create_dir_all(&package_path).map_err(AocIoError::CreateDirError)?;

    let template_dir = template_dir.as_ref().map(PathBuf::from);
    let template_path = template_dir.as_deref();
    let handle_file =
        |file_name: &str, fallback: fn(&Path, &str) -> AocResult<()>| -> AocResult<()> {
            let dest = package_path.join(file_name);
            if let Some(dir) = template_path {
                let src = dir.join(file_name);
                if src.exists() {
                    return copy_file_from_template(&src, &dest);
                }
            }

            fallback(&dest, package_name)
        };

    handle_file(CARGO_FILE, write_default_exercise_cargo)?;
    handle_file(MAIN_FILE, write_default_main_file)?;
    handle_file(LIB_FILE, write_default_lib_file)?;

    Ok(())
}

fn write_root_cargo(cargo_toml_path: &Path) -> AocResult<()> {
    let mut doc = DocumentMut::new();
    doc["package"] = table();
    doc["package"]["name"] = value("aocsuite-rust");
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    write_file(cargo_toml_path, doc.to_string())?;
    Ok(())
}

// only init if template doesnt exist.
fn write_default_exercise_cargo(cargo_toml_path: &Path, package_name: &str) -> AocResult<()> {
    let mut doc = DocumentMut::new();
    doc["package"] = table();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    write_file(cargo_toml_path, doc.to_string())?;
    Ok(())
}
fn write_default_main_file(cargo_toml_path: &Path, package_name: &str) -> AocResult<()> {
    let mut doc = DocumentMut::new();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    write_file(cargo_toml_path, doc.to_string())?;
    Ok(())
}

fn write_default_lib_file(cargo_toml_path: &Path, package_name: &str) -> AocResult<()> {
    let mut doc = DocumentMut::new();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    write_file(cargo_toml_path, doc.to_string())?;
    Ok(())
}

fn package_path_from_root(day: PuzzleDay, year: PuzzleYear) -> PathBuf {
    PathBuf::from(format!("year{year}")).join(format!("day{day}"))
}
