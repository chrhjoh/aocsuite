use crate::AocClientResult;
use aocsuite_utils::{Exercise, PuzzleDay, PuzzleYear};
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
        //TODO: Check string for error
        Ok(response.text()?)
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
        Ok(response.text()?)
    }
}
