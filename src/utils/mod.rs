use chrono::{DateTime, Datelike, TimeZone, Utc};
use chrono_tz::{Tz, US::Eastern};
use clap::{Subcommand, ValueEnum};

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

pub fn is_puzzle_released(day: PuzzleDay, year: PuzzleYear) -> bool {
    if day < 1 || day > 25 || year < 2015 {
        return false;
    }
    let now_utc = Utc::now();
    let now_eastern = now_utc.with_timezone(&Eastern);

    // save to unwrap. all days 1 to 25 dec are valid
    let release_date = Eastern.with_ymd_and_hms(year, 12, day, 0, 0, 0).unwrap();

    now_eastern >= release_date
}
pub fn is_year_released(day: PuzzleDay, year: PuzzleYear) -> bool {
    let now_utc = Utc::now();
    let now_eastern = now_utc.with_timezone(&Eastern);
    let now_year = now_eastern.year();

    if year < 2015 {
        false
    } else if year > now_year {
        false
    } else if year == now_year {
        // save to unwrap. all days 1 to 25 dec are valid
        let release_date = Eastern.with_ymd_and_hms(year, 12, day, 0, 0, 0).unwrap();
        now_eastern >= release_date
    } else {
        true
    }
}

pub fn today() -> DateTime<Tz> {
    let now_utc = Utc::now();
    now_utc.with_timezone(&Eastern)
}
