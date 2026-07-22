mod http_ansicalendar;
mod http_markdown;
mod http_submission;

pub use http_ansicalendar::{
    Calendar, CalendarCell, CalendarRow, CalendarStars, Rgb, parse_calendar,
};
pub use http_markdown::parse_puzzle_markdown;
pub use http_submission::{AocSubmissionResult, parse_submission};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("puzzle response did not contain an article")]
    MissingPuzzleArticle,
    #[error("calendar response did not contain a calendar")]
    MissingCalendar,
    #[error("submission response did not contain an article")]
    MissingSubmissionArticle,
}

pub type ParserResult<T> = Result<T, ParserError>;
