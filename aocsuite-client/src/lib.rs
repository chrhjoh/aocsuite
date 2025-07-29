use std::process::Command;

use aocsuite_config::{AocConfigError, ConfigOpt, get_config_val};
use aocsuite_utils::{Exercise, PuzzleDay, PuzzleYear};
use reqwest::blocking::{Client, Response};
use reqwest::header::{COOKIE, HeaderMap};
use thiserror::Error;

const BASE_URL: &str = "https://adventofcode.com";

#[derive(Debug, Clone, Copy)]
pub enum AocPage {
    Puzzle(PuzzleDay, PuzzleYear),
    Input(PuzzleDay, PuzzleYear),
    Submit(PuzzleDay, PuzzleYear),
    Calendar(PuzzleYear),
    Leaderboard(PuzzleYear, Option<u32>),
}

impl ToString for AocPage {
    fn to_string(&self) -> String {
        match self {
            AocPage::Puzzle(day, year) => format!("{}/{}/day/{}", BASE_URL, year, day),
            AocPage::Input(day, year) => {
                format!("{}/{}/day/{}/input", BASE_URL, year, day)
            }
            AocPage::Submit(day, year) => {
                format!("{}/{}/day/{}/answer", BASE_URL, year, day)
            }
            AocPage::Calendar(year) => format!("{}/{}", BASE_URL, year),
            AocPage::Leaderboard(year, id) => match id {
                Some(id) => format!("{}/{}/leaderboard/private/view/{}", BASE_URL, year, id),
                None => format!("{}/{}/leaderboard", BASE_URL, year),
            },
        }
    }
}

fn build_http_client() -> AocClientResult<Client> {
    let session: String = get_config_val(&ConfigOpt::Session, None, None)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        format!("session={}", session)
            .parse()
            .expect("Session can be parsed into String"),
    );
    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
}

pub fn download_file(page: &AocPage) -> AocClientResult<String> {
    let client = build_http_client()?;
    let response: Response = client
        .get(&page.to_string())
        .send()
        .map_err(|e| AocClientError::Http(e))?;
    //TODO: use the text to improve error handling from the response
    Ok(response.text().map_err(|e| AocClientError::Http(e))?)
}

pub fn open_page(page: &AocPage) -> AocClientResult<()> {
    let url = page.to_string();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(&url).status();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(&url).status();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").args(["/C", "start", &url]).status();

    result?;
    Ok(())
}
pub fn post_answer(
    answer: &str,
    level: &Exercise,
    day: PuzzleDay,
    year: PuzzleYear,
) -> AocClientResult<String> {
    let params = [("level", level.to_string()), ("answer", answer.to_string())];
    let page = AocPage::Submit(day, year).to_string();
    let client = build_http_client()?;
    let response: Response = client.post(&page).form(&params).send()?;
    let answer = response.text()?;
    Ok(answer)
}

pub type AocClientResult<T> = Result<T, AocClientError>;

#[derive(Debug, Error)]
pub enum AocClientError {
    #[error("Http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("UnreleasedError: {0}")]
    Unreleased(#[from] aocsuite_utils::ReleaseError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTML parsing error: {0}")]
    HtmlError(String),

    #[error("AoC session error: {0}")]
    Session(String),

    #[error(transparent)]
    Config(#[from] AocConfigError),
}
