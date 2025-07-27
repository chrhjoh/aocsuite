use std::path::PathBuf;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::{Tz, US::Eastern};
use clap::ValueEnum;
use thiserror::Error;

pub type PuzzleDay = u32;
pub type PuzzleYear = i32;

#[derive(Debug, Clone, ValueEnum, PartialEq, Eq)]
pub enum Exercise {
    #[clap(alias = "1")]
    One,
    #[clap(alias = "2")]
    Two,
}
impl ToString for Exercise {
    fn to_string(&self) -> String {
        match self {
            Exercise::One => "1".to_string(),
            Exercise::Two => "2".to_string(),
        }
    }
}

type AocReleaseResult<T> = Result<T, PuzzleNotReleasedError>;

#[derive(Debug, Error)]
#[error("Puzzle for {day} {year} has not been released yet.")]
pub struct PuzzleNotReleasedError {
    pub day: PuzzleDay,
    pub year: PuzzleYear,
}

pub fn valid_puzzle_release(day: PuzzleDay, year: PuzzleYear) -> AocReleaseResult<()> {
    if day < 1 || day > 25 || year < 2015 {
        return Err(PuzzleNotReleasedError { day, year });
    }
    let now_utc = Utc::now();
    let now_eastern = now_utc.with_timezone(&Eastern);

    // save to unwrap. all days 1 to 25 dec are valid
    let release_date = Eastern.with_ymd_and_hms(year, 12, day, 0, 0, 0).unwrap();

    if now_eastern >= release_date {
        Ok(())
    } else {
        return Err(PuzzleNotReleasedError { day, year });
    }
}
pub fn valid_year_release(day: PuzzleDay, year: PuzzleYear) -> AocReleaseResult<()> {
    let now_utc = Utc::now();
    let now_eastern = now_utc.with_timezone(&Eastern);
    let now_year = now_eastern.year();

    if year < 2015 || year > now_year {
        return Err(PuzzleNotReleasedError { day, year });
    } else if year == now_year {
        // save to unwrap. all days 1 to 25 dec are valid
        let release_date = Eastern.with_ymd_and_hms(year, 12, day, 0, 0, 0).unwrap();
        if now_eastern < release_date {
            return Err(PuzzleNotReleasedError { day, year });
        }
    }
    Ok(())
}

pub fn today() -> DateTime<Tz> {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern)
}
pub fn today_day() -> PuzzleDay {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern).day()
}
pub fn today_year() -> PuzzleYear {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern).year()
}

pub fn resolve_aocsuite_dir() -> PathBuf {
    let base = if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data_home)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local").join("share")
    } else {
        panic!("Neither XDG_DATA_HOME nor HOME environment variables are set")
    };
    base.join("aocsuite")
}
