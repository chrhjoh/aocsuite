//TODO: Inspiration from https://github.com/coriolinus/adventofcode-2020/blob/master/day-template/src/main.rs
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use toml_edit::DocumentMut;
use toml_edit::array;
use toml_edit::table;
use toml_edit::value;

use crate::utils::copy_file_from_template;
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
    fn scaffold(&self, day: PuzzleDay, year: PuzzleYear, template_dir: Option<String>) {
        fs::create_dir_all(&self.root_dir).expect("base directory should be writable");
        let root_cargo_path = self.root_dir.join(CARGO_FILE);
        match update_root_cargo(&root_cargo_path, day, year) {
            Ok(_) => {}
            Err(CargoErr::Io(err)) => {
                eprintln!(
                    "Error reading or writing root cargo file: {}. Did not modify root Cargo.toml",
                    err
                )
            }
            Err(CargoErr::Parse(err)) => {
                eprintln!(
                    "Error parsing Toml file: {}. Did not modify root Cargo.toml",
                    err
                )
            }
            Err(CargoErr::PathErr) => {
                eprintln!("Error creating path. Did not modify root Cargo.toml")
            }
        }
        let package_name = package_path_from_root(day, year);
        let package_path = self.root_dir.join(&package_name);

        if let Err(err) = create_exercise_package(
            &package_path,
            &package_name.to_str().expect("Path name should be valid"),
            template_dir,
        ) {
            eprintln!("Error occured while creating exercise package:{}", err);
        }
    }
    fn compile(&self, day: PuzzleDay, year: PuzzleYear) -> Option<io::Result<Output>> {
        let package_name = package_path_from_root(day, year);
        let output = Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg(package_name)
            .current_dir(&self.root_dir)
            .output();
        Some(output)
    }
    fn run(&self, day: PuzzleDay, year: PuzzleYear) -> io::Result<Output> {
        let package_name = package_path_from_root(day, year);
        let binary_path = self
            .root_dir
            .join("target")
            .join("debug")
            .join(package_name);
        Command::new(binary_path)
            .current_dir(&self.root_dir)
            .output()
    }
}

enum CargoErr {
    Io(std::io::Error),
    Parse(toml_edit::TomlError),
    PathErr,
}

fn update_root_cargo(
    root_cargo_path: &PathBuf,
    day: PuzzleDay,
    year: PuzzleYear,
) -> Result<(), CargoErr> {
    // check if cargo.toml exists. if not create it, then add the member to it.
    if !root_cargo_path.exists() {
        write_root_cargo(&root_cargo_path).map_err(CargoErr::Io)?;
    }
    // read the root cargo file and add member to it.
    let contents = fs::read_to_string(&root_cargo_path);
    let mut doc = contents
        .map_err(CargoErr::Io)?
        .parse::<DocumentMut>()
        .map_err(CargoErr::Parse)?;
    let workspace = doc["workspace"].or_insert(table());
    let members = &mut workspace["members"].or_insert(array());

    // create member file in new function (probably need a LanguageScaffold trait)
    let package_path = package_path_from_root(day, year);
    let package_name = match package_path.to_str() {
        Some(p) => p,
        None => return Err(CargoErr::PathErr),
    };

    if let Some(array) = members.as_array_mut() {
        if !array.iter().any(|v| v.as_str() == Some(package_name)) {
            array.push(package_name);
        }
    }
    Ok(())
}

fn create_exercise_package(
    package_path: &PathBuf,
    package_name: &str,
    template_dir: Option<String>,
) -> io::Result<()> {
    fs::create_dir_all(&package_path)?;
    if let Some(template) = template_dir {
        let template_dir = PathBuf::from(template);

        let cargo_template = template_dir.join(CARGO_FILE);
        let cargo_package = package_path.join(CARGO_FILE);
        if cargo_template.exists() {
            copy_file_from_template(&cargo_template, &cargo_package)?
        } else {
            write_default_exercise_cargo(&cargo_package, package_name)?;
        }
        let main_template = template_dir.join(MAIN_FILE);
        let main_package = package_path.join(MAIN_FILE);
        if main_template.exists() {
            copy_file_from_template(&main_template, &main_package)?
        } else {
            write_default_main_file(&main_package, package_name)?;
        }
        let lib_template = template_dir.join(LIB_FILE);
        let lib_package = package_path.join(LIB_FILE);
        if lib_template.exists() {
            copy_file_from_template(&lib_template, &lib_package)?
        } else {
            write_default_lib_file(&lib_package, package_name)?;
        }
    }

    Ok(())
}

fn write_root_cargo(cargo_toml_path: &PathBuf) -> io::Result<()> {
    let mut doc = DocumentMut::new();
    doc["package"]["name"] = value("aocsuite-rust");
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    doc["workspace"]["members"] = array();
    fs::write(cargo_toml_path, doc.to_string())?;
    Ok(())
}

// only init if template doesnt exist.
fn write_default_exercise_cargo(cargo_toml_path: &PathBuf, package_name: &str) -> io::Result<()> {
    let mut doc = DocumentMut::new();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    fs::write(cargo_toml_path, doc.to_string())?;
    Ok(())
}
fn write_default_main_file(cargo_toml_path: &PathBuf, package_name: &str) -> io::Result<()> {
    let mut doc = DocumentMut::new();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    fs::write(cargo_toml_path, doc.to_string())?;
    Ok(())
}

fn write_default_lib_file(cargo_toml_path: &PathBuf, package_name: &str) -> io::Result<()> {
    let mut doc = DocumentMut::new();
    doc["package"]["name"] = value(package_name);
    doc["package"]["version"] = value("0.1.0");
    doc["package"]["edition"] = value("2021");

    fs::write(cargo_toml_path, doc.to_string())?;
    Ok(())
}

fn package_path_from_root(day: PuzzleDay, year: PuzzleYear) -> PathBuf {
    PathBuf::from(format!("year{year}")).join(format!("day{day}"))
}
