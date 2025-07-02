use std::process::Command;

use crate::{AocClientResult, ParserType, parse_html};
use aocsuite_utils::{Exercise, PuzzleDay, PuzzleYear};
use regex::Regex;
use reqwest::blocking::{Client, Response};
use reqwest::header::{COOKIE, HeaderMap};

const BASE_URL: &str = "https://adventofcode.com";

#[derive(Debug, Clone, Copy)]
pub enum AocPage {
    Puzzle(PuzzleDay, PuzzleYear),
    Input(PuzzleDay, PuzzleYear),
    Submit(PuzzleDay, PuzzleYear),
    Calendar(PuzzleYear),
}

impl ToString for AocPage {
    fn to_string(&self) -> String {
        match self {
            AocPage::Puzzle(day, year) => format!("{}/{}/day/{}", BASE_URL, year, day),
            AocPage::Input(day, year) => {
                format!("https://adventofcode.com/{}/day/{}/input", year, day)
            }
            AocPage::Submit(day, year) => {
                format!("https://adventofcode.com/{}/day/{}/answer", year, day)
            }
            AocPage::Calendar(year) => format!("https://adventofcode.com/{}", year),
        }
    }
}

pub struct AocHttp {
    client: Client,
}

impl AocHttp {
    pub fn new(session: &str) -> AocClientResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            format!("session={}", session)
                .parse()
                .expect("Session can be parsed into String"),
        );
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self { client })
    }

    pub fn get(&self, page: AocPage) -> AocClientResult<String> {
        let response: Response = self.client.get(&page.to_string()).send()?;
        //TODO: could improve error handling from the response
        Ok(response.text()?)
    }

    pub fn get_cleaned(&self, page: AocPage, parser: ParserType) -> AocClientResult<String> {
        let calendar = self.get(page)?;
        let calendar = parse_html(&calendar, parser);
        Ok(calendar)
    }

    pub fn post_answer(
        &self,
        answer: &str,
        level: Exercise,
        day: PuzzleDay,
        year: PuzzleYear,
    ) -> AocClientResult<String> {
        let params = [("level", level.to_string()), ("answer", answer.to_string())];
        let page = AocPage::Submit(day, year).to_string();
        let response: Response = self.client.post(&page).form(&params).send()?;
        let mut response = parse_html(&response.text()?, ParserType::Markdown);
        response = remove_markdown_links(&response);
        //TODO: could improve response handling via errors and custom enum

        Ok(response)
    }
}

pub fn open_puzzle_page(day: PuzzleDay, year: PuzzleYear) -> AocClientResult<()> {
    let url = AocPage::Puzzle(day, year).to_string();

    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(&url).status();

    #[cfg(target_os = "linux")]
    let result = Command::new("xdg-open").arg(&url).status();

    #[cfg(target_os = "windows")]
    let result = Command::new("cmd").args(["/C", "start", &url]).status();

    result?;
    Ok(())
}

fn remove_markdown_links(text: &str) -> String {
    let re = Regex::new(r"\[\[?([^\[\]]+)\]?\]\([^)]+\)").unwrap();
    re.replace_all(text, "$1").into_owned()
}
