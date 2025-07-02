use std::fs;
use std::io;
use std::path::Path;

use aocsuite_utils::PuzzleDay;
use aocsuite_utils::PuzzleYear;

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

//TODO: Add option for templating later
pub fn copy_file_from_template(template_file: &Path, output_file: &Path) -> io::Result<()> {
    let content = fs::read_to_string(template_file)?;
    fs::write(output_file, content)
}
