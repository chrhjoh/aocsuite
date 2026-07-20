use std::process::Command;

use aocsuite_config::{AocConfigError, ConfigOpt, get_config_val};
use aocsuite_utils::{Exercise, PuzzleDay, PuzzleYear};
use reqwest::blocking::{Client, Response};
use reqwest::header::{COOKIE, HeaderMap, HeaderValue};
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
        self.url(BASE_URL)
    }
}

impl AocPage {
    fn url(&self, base_url: &str) -> String {
        let base_url = base_url.trim_end_matches('/');
        match self {
            AocPage::Puzzle(day, year) => format!("{base_url}/{year}/day/{day}"),
            AocPage::Input(day, year) => format!("{base_url}/{year}/day/{day}/input"),
            AocPage::Submit(day, year) => format!("{base_url}/{year}/day/{day}/answer"),
            AocPage::Calendar(year) => format!("{base_url}/{year}"),
            AocPage::Leaderboard(year, id) => match id {
                Some(id) => format!("{base_url}/{year}/leaderboard/private/view/{id}"),
                None => format!("{base_url}/{year}/leaderboard"),
            },
        }
    }
}

fn build_http_client() -> AocClientResult<Client> {
    let session: String = get_config_val(&ConfigOpt::Session, None, None)?;
    build_http_client_with_session(&session)
}

fn build_http_client_with_session(session: &str) -> AocClientResult<Client> {
    let mut headers = HeaderMap::new();
    let session_header = format!("session={session}")
        .parse::<HeaderValue>()
        .map_err(|error| AocClientError::Session(error.to_string()))?;
    headers.insert(COOKIE, session_header);
    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
}

pub fn download_file(page: &AocPage) -> AocClientResult<String> {
    let client = build_http_client()?;
    download_file_from(page, &client, BASE_URL)
}

fn download_file_from(page: &AocPage, client: &Client, base_url: &str) -> AocClientResult<String> {
    let response = client
        .get(page.url(base_url))
        .send()
        .map_err(|e| AocClientError::Http(e))?;
    parse_submission_response(response)
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
    let response = client.post(&page).form(&params).send()?;
    parse_submission_response(response)
}

fn parse_submission_response(response: Response) -> AocClientResult<String> {
    let status = response.status();
    if !status.is_success() {
        return Err(match status.as_u16() {
            401 | 403 => AocClientError::Authentication,
            429 => AocClientError::RateLimited,
            status => AocClientError::HttpStatus(status),
        });
    }

    let body = response.text()?;
    if body.contains("Please log in") {
        return Err(AocClientError::Authentication);
    }
    Ok(body)
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

    #[error("AoC authentication failed")]
    Authentication,

    #[error("AoC rate limited the request")]
    RateLimited,

    #[error("AoC returned HTTP status {0}")]
    HttpStatus(u16),

    #[error(transparent)]
    Config(#[from] AocConfigError),
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use reqwest::blocking::Client;

    use super::{AocClientError, AocPage, build_http_client_with_session, download_file_from};

    fn serve_once(status: u16, body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read test server address");
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept test request");
            let mut request = [0; 1024];
            stream.read(&mut request).expect("read test request");
            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .expect("write test response");
        });
        format!("http://{address}")
    }

    #[test]
    fn invalid_session_header_returns_a_typed_error() {
        assert!(matches!(
            build_http_client_with_session("valid\r\ninvalid"),
            Err(AocClientError::Session(_))
        ));
    }

    #[test]
    fn valid_session_header_builds_a_client() {
        assert!(build_http_client_with_session("valid-session").is_ok());
    }

    #[test]
    fn failed_http_statuses_return_typed_errors() {
        let client = Client::new();
        let page = AocPage::Puzzle(1, 2024);

        for (status, expected) in [
            (302, "status"),
            (400, "status"),
            (401, "authentication"),
            (429, "rate-limit"),
            (500, "status"),
        ] {
            let base_url = serve_once(status, "failure");
            let error = download_file_from(&page, &client, &base_url).expect_err("request fails");
            match expected {
                "authentication" => assert!(matches!(error, AocClientError::Authentication)),
                "rate-limit" => assert!(matches!(error, AocClientError::RateLimited)),
                _ => assert!(matches!(error, AocClientError::HttpStatus(code) if code == status)),
            }
        }
    }

    #[test]
    fn successful_login_page_returns_an_authentication_error() {
        let base_url = serve_once(200, "Please log in to get your puzzle input.");
        let error = download_file_from(&AocPage::Input(1, 2024), &Client::new(), &base_url)
            .expect_err("login page is rejected");

        assert!(matches!(error, AocClientError::Authentication));
    }
}
