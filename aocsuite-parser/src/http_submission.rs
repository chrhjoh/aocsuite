use regex::Regex;

use crate::parse;

#[derive(Debug, PartialEq, Eq)]
pub enum AocSubmissionResult {
    Correct,
    AlreadyCompleted,
    IncorrectTooHigh,
    IncorrectTooLow,
    Incorrect,
    RateLimited(u64), // seconds to wait
    Locked,
    EmptySubmission,
    InvalidFormat,
    Unknown(String),
}
impl std::fmt::Display for AocSubmissionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AocSubmissionResult::Correct => write!(f, "✅ Correct! That's the right answer!"),
            AocSubmissionResult::AlreadyCompleted => {
                write!(f, "ℹ️  You've already completed this puzzle.")
            }
            AocSubmissionResult::IncorrectTooHigh => write!(f, "❌ Your answer is too high."),
            AocSubmissionResult::IncorrectTooLow => write!(f, "❌ Your answer is too low."),
            AocSubmissionResult::Incorrect => write!(f, "❌ That's not the right answer."),
            AocSubmissionResult::RateLimited(seconds) => {
                write!(
                    f,
                    "⏳ Rate limited. Please wait {} seconds before submitting again.",
                    seconds
                )
            }
            AocSubmissionResult::Locked => {
                write!(f, "🔒 This part of the puzzle is not yet unlocked.")
            }
            AocSubmissionResult::EmptySubmission => write!(f, "⚠️  You didn't provide an answer."),
            AocSubmissionResult::InvalidFormat => {
                write!(f, "⚠️  Your answer isn't in the expected format.")
            }
            AocSubmissionResult::Unknown(msg) => write!(f, "❓ Unknown response: {}", msg),
        }
    }
}

pub fn parse_submission_result(text: &str) -> AocSubmissionResult {
    let markdown = parse(text, crate::ParserType::MarkdownArticle);

    if markdown.contains("That's the right answer!") {
        AocSubmissionResult::Correct
    } else if markdown.contains("You've already completed this puzzle")
        || markdown.contains("You don't need to guess; you've already completed this puzzle.")
    {
        AocSubmissionResult::AlreadyCompleted
    } else if markdown.contains("too high") {
        AocSubmissionResult::IncorrectTooHigh
    } else if markdown.contains("too low") {
        AocSubmissionResult::IncorrectTooLow
    } else if markdown.contains("That's not the right answer") {
        AocSubmissionResult::Incorrect
    } else if let Some(wait_secs) = extract_wait_time(&markdown) {
        AocSubmissionResult::RateLimited(wait_secs)
    } else if markdown.contains("haven't unlocked this part yet") {
        AocSubmissionResult::Locked
    } else if markdown.contains("did not provide an answer") {
        AocSubmissionResult::EmptySubmission
    } else if markdown.contains("isn't in the expected format") {
        AocSubmissionResult::InvalidFormat
    } else {
        AocSubmissionResult::Unknown(markdown.to_string())
    }
}
fn extract_wait_time(text: &str) -> Option<u64> {
    let re = Regex::new(r"you have to wait (\d+) seconds").ok()?;
    re.captures(text)
        .and_then(|caps| caps.get(1)?.as_str().parse().ok())
}
