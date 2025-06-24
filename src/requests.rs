use reqwest::blocking::{Client, Response};
use reqwest::header::{COOKIE, HeaderMap};

use crate::AocResult;
use crate::error::AocHttpError;

#[derive(Debug, Clone, Copy)]
pub enum AocPage {
    Puzzle,
    Input,
    Submit,
    Calendar,
}

struct AocURL {
    base: String,
}

impl AocURL {
    fn new() -> Self {
        Self {
            base: "https://adventofcode.com".to_string(),
        }
    }

    fn resolve(&self, year: u32, page: AocPage, day: Option<u32>) -> String {
        match page {
            AocPage::Puzzle => {
                let day = day.expect("Puzzle requires a day");
                format!("{}/{}/day/{}", self.base, year, day)
            }
            AocPage::Input => {
                let day = day.expect("Input requires a day");
                format!("{}/{}/day/{}/input", self.base, year, day)
            }
            AocPage::Submit => {
                let day = day.expect("Submit requires a day");
                format!("{}/{}/day/{}/answer", self.base, year, day)
            }
            AocPage::Calendar => format!("{}/{}", self.base, year),
        }
    }
}

pub struct AocHttp {
    client: Client,
    url: AocURL,
}

impl AocHttp {
    pub fn new(session: &str) -> AocResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            format!("session={}", session)
                .parse()
                .expect("Session can be parsed into String"),
        );
        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(AocHttpError::ClientError)?;
        Ok(Self {
            client,
            url: AocURL::new(),
        })
    }

    pub fn get(&self, year: u32, page: AocPage, day: Option<u32>) -> AocResult<String> {
        let url = self.url.resolve(year, page, day);
        let response: Response = self
            .client
            .get(&url)
            .send()
            .map_err(|e| AocHttpError::GetError(page, e))?;
        Ok(response
            .text()
            .map_err(|e| AocHttpError::ResponseError(page, e))?)
    }

    pub fn post_answer(&self, year: u32, day: u32, answer: &str, level: u32) -> AocResult<String> {
        let params = [("level", level.to_string()), ("answer", answer.to_string())];
        let url = self.url.resolve(year, AocPage::Submit, Some(day));
        let response: Response = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .map_err(|e| AocHttpError::PostError(AocPage::Submit, e))?;
        Ok(response
            .text()
            .map_err(|e| AocHttpError::ResponseError(AocPage::Submit, e))?)
    }
}

pub struct AocRequestParser {}
