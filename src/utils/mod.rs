use std::path::{Path, PathBuf};
use std::{fs, io};

use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::{Tz, US::Eastern};
use clap::{Subcommand, ValueEnum};

use crate::error::AocIoError;
use crate::{AocError, AocResult};

pub type PuzzleDay = u32;
pub type PuzzleYear = i32;

#[derive(Subcommand, Debug, Clone)]
pub enum DownloadMode {
    /// Download both puzzle description and input (default)
    All,

    /// Download only the puzzle input
    Input,

    /// Download only the puzzle description
    Puzzle,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum Exercise {
    #[value(name = "1")]
    One,

    #[value(name = "2")]
    Two,
}

const BASE_URL: &str = "https://adventofcode.com";

pub fn calendar_url(year: PuzzleYear) -> String {
    format!("{}/{}/calendar", BASE_URL, year)
}
pub fn puzzle_url(year: PuzzleYear, day: u32) -> String {
    format!("{}/{}/day/{}", BASE_URL, year, day)
}

pub fn submit_url(year: PuzzleYear, day: u32) -> String {
    format!("{}/{}/day/{}/answer", BASE_URL, year, day)
}

/// Returns public leaderboard URL or private leaderboard URL if id is specified
pub fn leaderboard_url(year: PuzzleYear, leaderboard_id: Option<u32>) -> String {
    let base_url = format!("{}/{}/leaderboard", BASE_URL, year);
    if let Some(leaderboard_id) = leaderboard_id {
        format!("{}/leaderboard/private/view/{}", base_url, leaderboard_id)
    } else {
        base_url
    }
}

#[derive(Debug)]
pub struct UnreleasedError {
    day: PuzzleDay,
    year: PuzzleYear,
}

impl std::fmt::Display for UnreleasedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Advent of code not released yet: day: {}, year {}",
            self.day, self.year
        )
    }
}

impl std::error::Error for UnreleasedError {}

pub fn valid_puzzle_release(day: PuzzleDay, year: PuzzleYear) -> Result<(), AocError> {
    if day < 1 || day > 25 || year < 2015 {
        return Err(AocError::Unreleased(day, year));
    }
    let now_utc = Utc::now();
    let now_eastern = now_utc.with_timezone(&Eastern);

    // save to unwrap. all days 1 to 25 dec are valid
    let release_date = Eastern.with_ymd_and_hms(year, 12, day, 0, 0, 0).unwrap();

    if now_eastern >= release_date {
        Ok(())
    } else {
        return Err(AocError::Unreleased(day, year));
    }
}
pub fn valid_year_release(day: PuzzleDay, year: PuzzleYear) -> Result<(), AocError> {
    let now_utc = Utc::now();
    let now_eastern = now_utc.with_timezone(&Eastern);
    let now_year = now_eastern.year();

    if year < 2015 || year > now_year {
        return Err(AocError::Unreleased(day, year));
    } else if year == now_year {
        // save to unwrap. all days 1 to 25 dec are valid
        let release_date = Eastern.with_ymd_and_hms(year, 12, day, 0, 0, 0).unwrap();
        if now_eastern < release_date {
            return Err(AocError::Unreleased(day, year));
        }
    }
    Ok(())
}

pub fn today() -> DateTime<Tz> {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern)
}

//TODO: Add option for templating later
pub fn copy_file_from_template(template_file: &Path, output_file: &Path) -> AocResult<()> {
    let content = fs::read_to_string(template_file).map_err(AocIoError::ReadError)?;
    write_file(output_file, content)
}

pub fn write_file(filename: &Path, content: String) -> AocResult<()> {
    fs::write(filename, content).map_err(AocIoError::WriteError)?;
    Ok(())
}

pub fn read_file(filename: &Path) -> AocResult<String> {
    let contents = fs::read_to_string(&filename).map_err(AocIoError::WriteError)?;
    Ok(contents)
}
pub fn create_dirs(dir: &Path) -> AocResult<()> {
    fs::create_dir_all(dir).map_err(AocIoError::CreateDirError)?;
    Ok(())
}
