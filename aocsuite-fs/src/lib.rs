use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use aocsuite_utils::PuzzleDay;
use aocsuite_utils::PuzzleYear;
use thiserror::Error;

pub enum AocDataFile {
    Input(PuzzleDay, PuzzleYear),
    Puzzle(PuzzleDay, PuzzleYear),
    Example(PuzzleDay, PuzzleYear),
}

impl ToString for AocDataFile {
    fn to_string(&self) -> String {
        let (day, year, file) = match self {
            AocDataFile::Input(day, year) => (day, year, "input.txt"),
            AocDataFile::Puzzle(day, year) => (day, year, "puzzle.md"),
            AocDataFile::Example(day, year) => (day, year, "example.txt"),
        };
        let dir = AocDataDir::new(*day, *year);
        format!("{}/{}", dir.to_string(), file)
    }
}

pub struct AocDataDir {
    day: PuzzleDay,
    year: PuzzleYear,
}

impl AocDataDir {
    pub fn new(day: PuzzleDay, year: PuzzleYear) -> AocDataDir {
        AocDataDir { day, year }
    }
}

impl ToString for AocDataDir {
    fn to_string(&self) -> String {
        format!("data/year{}/day{}", self.year, self.day)
    }
}

pub fn copy_file_from_template(template_file: &Path, output_file: &Path) -> io::Result<()> {
    let content = fs::read_to_string(template_file)?;
    fs::write(output_file, content)
}

type AocFileResult<T> = Result<T, AocFileError>;

#[derive(Error, Debug)]
pub enum AocFileError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn ensure_files_exist<I, P>(paths: I) -> AocFileResult<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    for path in paths {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(AocFileError::FileNotFound(path_ref.display().to_string()));
        }
    }
    Ok(())
}

pub fn confirm_overwrite_if_exists(path: &Path, overwrite: bool) -> AocFileResult<bool> {
    if path.exists() & !overwrite {
        print!(
            "Warning: File '{}' already exists.\nDo you want to overwrite it? (y/N): ",
            path.display()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let answer = input.trim().to_lowercase();
        if answer == "y" {
            println!("Overwriting file.");
            Ok(true)
        } else {
            println!("Operation cancelled.");
            Ok(false)
        }
    } else {
        // File doesn't exist, so no confirmation needed
        Ok(true)
    }
}

pub fn write_with_confirmation<P: AsRef<Path>>(
    path: P,
    contents: String,
    overwrite: bool,
) -> AocFileResult<()> {
    let path = path.as_ref();
    if confirm_overwrite_if_exists(path, overwrite)? {
        std::fs::write(path, contents)?;
        println!("{} saved successfully.", path.display());
        Ok(())
    } else {
        println!("File not saved.");
        Ok(())
    }
}
