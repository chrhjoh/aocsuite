use std::{thread, time::Duration};

use aocsuite_utils::{PuzzleId, PuzzlePart, PuzzleYear};
use reqwest::{
    blocking::{Client, Response},
    header::{COOKIE, HeaderMap, HeaderValue, RETRY_AFTER},
    redirect::Policy,
};
use thiserror::Error;

const BASE_URL: &str = "https://adventofcode.com";
const USER_AGENT: &str = concat!("aocsuite/", env!("CARGO_PKG_VERSION"));
const GET_RETRY_ATTEMPTS: u32 = 3;
const INITIAL_GET_RETRY_BACKOFF: Duration = Duration::from_secs(1);
const MAX_RETRY_AFTER: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy)]
pub enum AocPage {
    Puzzle(PuzzleId),
    Input(PuzzleId),
    Submit(PuzzleId),
    Calendar(PuzzleYear),
    Leaderboard(PuzzleYear, Option<u32>),
}

impl AocPage {
    fn url(&self, base_url: &str) -> String {
        let base_url = base_url.trim_end_matches('/');
        match self {
            Self::Puzzle(puzzle) => format!("{base_url}/{}/day/{}", puzzle.year, puzzle.day),
            Self::Input(puzzle) => format!("{base_url}/{}/day/{}/input", puzzle.year, puzzle.day),
            Self::Submit(puzzle) => format!("{base_url}/{}/day/{}/answer", puzzle.year, puzzle.day),
            Self::Calendar(year) => format!("{base_url}/{year}"),
            Self::Leaderboard(year, id) => match id {
                Some(id) => format!("{base_url}/{year}/leaderboard/private/view/{id}"),
                None => format!("{base_url}/{year}/leaderboard"),
            },
        }
    }

    fn requires_session(&self) -> bool {
        matches!(
            self,
            Self::Input(_) | Self::Submit(_) | Self::Leaderboard(_, Some(_))
        )
    }
}

impl std::fmt::Display for AocPage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.url(BASE_URL))
    }
}

#[derive(Debug, Clone)]
pub struct AocClientOptions {
    pub base_url: String,
    pub timeout: Duration,
    pub user_agent: String,
}

impl Default for AocClientOptions {
    fn default() -> Self {
        Self {
            base_url: BASE_URL.to_owned(),
            timeout: Duration::from_secs(30),
            user_agent: USER_AGENT.to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AocClient {
    client: Client,
    base_url: String,
    has_session: bool,
    _sleep: fn(Duration),
}

impl AocClient {
    pub fn new(session: Option<&str>, options: AocClientOptions) -> AocClientResult<Self> {
        let mut headers = HeaderMap::new();
        if let Some(session) = session {
            if session.is_empty() {
                return Err(AocClientError::Session(
                    "session token cannot be empty".to_owned(),
                ));
            }
            let session_header = format!("session={session}")
                .parse::<HeaderValue>()
                .map_err(|error| AocClientError::Session(error.to_string()))?;
            headers.insert(COOKIE, session_header);
        }

        let client = Client::builder()
            .default_headers(headers)
            .timeout(options.timeout)
            .user_agent(options.user_agent)
            .redirect(Policy::none())
            .build()?;
        Ok(Self {
            client,
            base_url: options.base_url.trim_end_matches('/').to_owned(),
            has_session: session.is_some(),
            _sleep: thread::sleep,
        })
    }

    pub fn download(&self, page: &AocPage) -> AocClientResult<String> {
        self.ensure_session(page)?;
        let url = page.url(&self.base_url);
        let mut sleep_timer;

        for retry in 0..=GET_RETRY_ATTEMPTS {
            match self.client.get(&url).send() {
                Ok(response) => match parse_response(response) {
                    Ok(body) => return Ok(body),
                    Err(AocClientError::RateLimited(delay)) if retry < GET_RETRY_ATTEMPTS => {
                        sleep_timer = delay.unwrap_or_else(|| default_retry_delay(retry));
                    }
                    Err(AocClientError::Http(_)) if retry < GET_RETRY_ATTEMPTS => {
                        sleep_timer = default_retry_delay(retry);
                    }
                    Err(error) => return Err(error),
                },
                Err(_) if retry < GET_RETRY_ATTEMPTS => {
                    sleep_timer = default_retry_delay(retry);
                }
                Err(error) => return Err(error.into()),
            }
            self.sleep(sleep_timer);
        }

        unreachable!("GET retry loop always returns on its final attempt")
    }
    fn sleep(&self, time: Duration) -> () {
        (self._sleep)(time)
    }

    pub fn submit(
        &self,
        puzzle: PuzzleId,
        part: PuzzlePart,
        answer: &str,
    ) -> AocClientResult<String> {
        let page = AocPage::Submit(puzzle);
        self.ensure_session(&page)?;
        let params = [("level", part.to_string()), ("answer", answer.to_owned())];
        let response = self
            .client
            .post(page.url(&self.base_url))
            .form(&params)
            .send()?;
        parse_response(response)
    }

    fn ensure_session(&self, page: &AocPage) -> AocClientResult<()> {
        if page.requires_session() && !self.has_session {
            Err(AocClientError::MissingSession)
        } else {
            Ok(())
        }
    }
}

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 429 | 500 | 502 | 503 | 504)
}

fn retry_delay(status: reqwest::StatusCode, headers: &HeaderMap) -> Option<Duration> {
    if matches!(status.as_u16(), 429 | 503) {
        return headers
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_secs)
            .map(|delay| delay.min(MAX_RETRY_AFTER));
    }

    None
}

fn default_retry_delay(retry: u32) -> Duration {
    INITIAL_GET_RETRY_BACKOFF.saturating_mul(1_u32 << retry)
}

fn parse_response(response: Response) -> AocClientResult<String> {
    let status = response.status();
    if !status.is_success() {
        if is_retryable_status(status) {
            return Err(AocClientError::RateLimited(retry_delay(
                status,
                response.headers(),
            )));
        }

        return Err(match status.as_u16() {
            401 | 403 => AocClientError::Authentication,
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
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("AoC session error: {0}")]
    Session(String),

    #[error("this AoC request requires a session token")]
    MissingSession,

    #[error("AoC authentication failed")]
    Authentication,

    #[error("AoC request can be retried")]
    RateLimited(Option<Duration>),

    #[error("AoC returned HTTP status {0}")]
    HttpStatus(u16),
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc::{self, Receiver},
        thread,
        time::Duration,
    };

    use aocsuite_utils::{PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear};

    use super::{
        AocClient, AocClientError, AocClientOptions, AocPage, GET_RETRY_ATTEMPTS,
        default_retry_delay, retry_delay,
    };
    use reqwest::{
        StatusCode,
        header::{HeaderMap, HeaderValue, RETRY_AFTER},
    };

    fn puzzle() -> PuzzleId {
        PuzzleId::new(
            PuzzleDay::new(1).expect("valid test day"),
            PuzzleYear::new(2024).expect("valid test year"),
        )
    }

    fn serve_once(status: u16, body: &'static str) -> (String, Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read test server address");
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept test request");
            let mut request = [0; 4096];
            let bytes = stream.read(&mut request).expect("read test request");
            let _ = sender.send(String::from_utf8_lossy(&request[..bytes]).into_owned());
            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .expect("write test response");
        });
        (format!("http://{address}"), receiver)
    }

    fn client(base_url: String, session: Option<&str>) -> AocClient {
        let mut client = AocClient::new(
            session,
            AocClientOptions {
                base_url,
                user_agent: "aocsuite-test/1".to_owned(),
                ..AocClientOptions::default()
            },
        )
        .expect("build test client");
        client._sleep = |_| {};
        client
    }

    fn serve_responses(
        responses: Vec<(u16, &'static str, Option<&'static str>)>,
    ) -> (String, Receiver<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read test server address");
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let mut requests = Vec::new();
            for (status, body, retry_after) in responses {
                let (mut stream, _) = listener.accept().expect("accept test request");
                let mut request = [0; 4096];
                let bytes = stream.read(&mut request).expect("read test request");
                requests.push(String::from_utf8_lossy(&request[..bytes]).into_owned());
                let retry_after = retry_after
                    .map(|value| format!("Retry-After: {value}\r\n"))
                    .unwrap_or_default();
                write!(
                    stream,
                    "HTTP/1.1 {status} Test\r\n{retry_after}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .expect("write test response");
            }
            sender.send(requests).expect("send requests");
        });
        (format!("http://{address}"), receiver)
    }

    fn serve_disconnections(count: u32) -> (String, Receiver<u32>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read test server address");
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            for _ in 0..count {
                let (mut stream, _) = listener.accept().expect("accept test request");
                let mut request = [0; 1024];
                let _ = stream.read(&mut request);
            }
            sender.send(count).expect("send request count");
        });
        (format!("http://{address}"), receiver)
    }

    fn serve_after(delay: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read test server address");
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept test request");
            let mut request = [0; 1024];
            let _ = stream.read(&mut request);
            thread::sleep(delay);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
        });
        format!("http://{address}")
    }

    #[test]
    fn invalid_session_header_returns_a_typed_error() {
        assert!(matches!(
            AocClient::new(Some("valid\r\ninvalid"), AocClientOptions::default()),
            Err(AocClientError::Session(_))
        ));
    }

    #[test]
    fn pages_build_every_supported_url_shape() {
        let puzzle = puzzle();
        let base = "https://example.com/root/";

        assert_eq!(
            AocPage::Puzzle(puzzle).url(base),
            "https://example.com/root/2024/day/1"
        );
        assert_eq!(
            AocPage::Input(puzzle).url(base),
            "https://example.com/root/2024/day/1/input"
        );
        assert_eq!(
            AocPage::Submit(puzzle).url(base),
            "https://example.com/root/2024/day/1/answer"
        );
        assert_eq!(
            AocPage::Calendar(puzzle.year).url(base),
            "https://example.com/root/2024"
        );
        assert_eq!(
            AocPage::Leaderboard(puzzle.year, None).url(base),
            "https://example.com/root/2024/leaderboard"
        );
        assert_eq!(
            AocPage::Leaderboard(puzzle.year, Some(42)).url(base),
            "https://example.com/root/2024/leaderboard/private/view/42"
        );
    }

    #[test]
    fn public_requests_allow_an_absent_session() {
        let (base_url, request) = serve_once(200, "puzzle");
        let client = client(base_url, None);
        let puzzle = puzzle();

        assert_eq!(client.download(&AocPage::Puzzle(puzzle)).unwrap(), "puzzle");
        let request = request.recv().unwrap();
        let request = request.to_ascii_lowercase();
        assert!(request.contains("user-agent: aocsuite-test/1"));
        assert!(!request.contains("cookie:"));
    }

    #[test]
    fn private_requests_require_a_session() {
        let client = client("http://127.0.0.1:1".to_owned(), None);
        let puzzle = puzzle();

        assert!(matches!(
            client.download(&AocPage::Input(puzzle)),
            Err(AocClientError::MissingSession)
        ));
        assert!(matches!(
            client.submit(puzzle, PuzzlePart::One, "answer"),
            Err(AocClientError::MissingSession)
        ));
    }

    #[test]
    fn request_timeout_is_configurable() {
        let options = AocClientOptions {
            base_url: serve_after(Duration::from_millis(250)),
            timeout: Duration::from_millis(20),
            ..AocClientOptions::default()
        };
        let mut client = AocClient::new(None, options).unwrap();
        client._sleep = |_| {};
        let puzzle = puzzle();

        assert!(matches!(
            client.download(&AocPage::Puzzle(puzzle)),
            Err(AocClientError::Http(error)) if error.is_timeout()
        ));
    }

    #[test]
    fn configured_sessions_are_attached_to_requests() {
        let (base_url, request) = serve_once(200, "input");
        let client = client(base_url, Some("test-session"));
        let puzzle = puzzle();

        client.download(&AocPage::Input(puzzle)).unwrap();
        assert!(
            request
                .recv()
                .unwrap()
                .to_ascii_lowercase()
                .contains("cookie: session=test-session")
        );
    }

    #[test]
    fn submissions_send_the_part_answer_and_session() {
        let (base_url, request) = serve_once(200, "correct");
        let client = client(base_url, Some("test-session"));

        assert_eq!(
            client.submit(puzzle(), PuzzlePart::Two, "12345").unwrap(),
            "correct"
        );
        let request = request.recv().unwrap();
        assert!(request.starts_with("POST /2024/day/1/answer HTTP/1.1"));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("cookie: session=test-session")
        );
        assert!(request.contains("level=2&answer=12345"));
    }

    #[test]
    fn submissions_are_not_retried_after_a_transient_status() {
        let (base_url, request) = serve_once(503, "try later");

        assert!(matches!(
            client(base_url, Some("test-session")).submit(puzzle(), PuzzlePart::One, "12345"),
            Err(AocClientError::RateLimited(None))
        ));
        assert!(
            request
                .recv()
                .unwrap()
                .starts_with("POST /2024/day/1/answer HTTP/1.1")
        );
    }

    #[test]
    fn failed_http_statuses_return_typed_errors() {
        let puzzle = puzzle();
        for (status, expected) in [(302, "status"), (400, "status"), (401, "authentication")] {
            let (base_url, _) = serve_once(status, "failure");
            let error = client(base_url, None)
                .download(&AocPage::Puzzle(puzzle))
                .expect_err("request fails");
            match expected {
                "authentication" => assert!(matches!(error, AocClientError::Authentication)),
                _ => assert!(matches!(error, AocClientError::HttpStatus(code) if code == status)),
            }
        }
    }

    #[test]
    fn transient_get_status_is_retried_until_a_response_succeeds() {
        let (base_url, requests) =
            serve_responses(vec![(500, "retry", None), (200, "puzzle", None)]);

        assert_eq!(
            client(base_url, None)
                .download(&AocPage::Puzzle(puzzle()))
                .unwrap(),
            "puzzle"
        );
        assert_eq!(requests.recv().unwrap().len(), 2);
    }

    #[test]
    fn exhausted_transient_get_status_returns_the_final_typed_error() {
        let (base_url, requests) = serve_responses(vec![(429, "retry", None); 4]);

        assert!(matches!(
            client(base_url, None).download(&AocPage::Puzzle(puzzle())),
            Err(AocClientError::RateLimited(None))
        ));
        assert_eq!(requests.recv().unwrap().len(), 4);
    }

    #[test]
    fn get_transport_errors_are_retried() {
        let (base_url, requests) = serve_disconnections(GET_RETRY_ATTEMPTS + 1);

        assert!(matches!(
            client(base_url, None).download(&AocPage::Puzzle(puzzle())),
            Err(AocClientError::Http(_))
        ));
        assert_eq!(requests.recv().unwrap(), GET_RETRY_ATTEMPTS + 1);
    }

    #[test]
    fn retry_after_is_used_only_for_rate_limited_and_unavailable_responses() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("90"));

        assert_eq!(
            retry_delay(StatusCode::TOO_MANY_REQUESTS, &headers),
            Some(Duration::from_secs(60))
        );
        assert_eq!(
            retry_delay(StatusCode::SERVICE_UNAVAILABLE, &headers),
            Some(Duration::from_secs(60))
        );
        assert_eq!(
            retry_delay(StatusCode::INTERNAL_SERVER_ERROR, &headers),
            None
        );
        assert_eq!(default_retry_delay(0), Duration::from_secs(1));
        assert_eq!(default_retry_delay(1), Duration::from_secs(2));
        assert_eq!(default_retry_delay(2), Duration::from_secs(4));
    }

    #[test]
    fn successful_login_page_returns_an_authentication_error() {
        let (base_url, _) = serve_once(200, "Please log in to get your puzzle input.");
        let puzzle = puzzle();
        let error = client(base_url, Some("expired"))
            .download(&AocPage::Input(puzzle))
            .expect_err("login page is rejected");

        assert!(matches!(error, AocClientError::Authentication));
    }
}
