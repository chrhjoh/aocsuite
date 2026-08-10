use regex::Regex;

use crate::{ParserError, ParserResult, http_markdown::parse_article_markdown};

#[derive(Debug, PartialEq, Eq)]
pub enum AocSubmissionResult {
    Correct,
    AlreadyCompleted,
    IncorrectTooHigh,
    IncorrectTooLow,
    Incorrect,
    RateLimited(u64),
    Locked,
    EmptySubmission,
    InvalidFormat,
    Unknown(String),
}

pub fn parse_submission(html: &str) -> ParserResult<AocSubmissionResult> {
    let markdown = parse_article_markdown(html);
    if markdown.trim().is_empty() {
        return Err(ParserError::MissingSubmissionArticle);
    }

    Ok(if markdown.contains("That's the right answer!") {
        AocSubmissionResult::Correct
    } else if markdown.contains("You've already completed this puzzle")
        || markdown.contains("You don't need to guess; you've already completed this puzzle.")
    {
        AocSubmissionResult::AlreadyCompleted
    } else if let Some(wait_secs) = extract_wait_time(&markdown) {
        AocSubmissionResult::RateLimited(wait_secs)
    } else if markdown.contains("too high") {
        AocSubmissionResult::IncorrectTooHigh
    } else if markdown.contains("too low") {
        AocSubmissionResult::IncorrectTooLow
    } else if markdown.contains("That's not the right answer") {
        AocSubmissionResult::Incorrect
    } else if markdown.contains("haven't unlocked this part yet") {
        AocSubmissionResult::Locked
    } else if markdown.contains("did not provide an answer") {
        AocSubmissionResult::EmptySubmission
    } else if markdown.contains("isn't in the expected format") {
        AocSubmissionResult::InvalidFormat
    } else {
        AocSubmissionResult::Unknown(markdown)
    })
}

fn extract_wait_time(text: &str) -> Option<u64> {
    let re = Regex::new(
        r"(?i)\b(?:you\s+have(?:\s+to\s+wait)?|please\s+wait)\s+(?:(\d+)\s*(?:minutes?|mins?|m)\b(?:\s*(?:and\s*)?(\d+)\s*(?:seconds?|secs?|s)\b)?|(\d+)\s*(?:seconds?|secs?|s)\b)",
    )
    .ok()?;
    let caps = re.captures(text)?;
    let minutes = caps
        .get(1)
        .and_then(|value| value.as_str().parse::<u64>().ok())
        .unwrap_or(0);
    let seconds = caps
        .get(2)
        .or_else(|| caps.get(3))
        .and_then(|value| value.as_str().parse::<u64>().ok())
        .unwrap_or(0);

    minutes.checked_mul(60)?.checked_add(seconds)
}

#[cfg(test)]
mod tests {
    use super::extract_wait_time;

    #[test]
    fn extracts_aoc_wait_times() {
        let cases = [
            ("you have to wait 12 seconds.", Some(12)),
            ("You have to wait 1 second!", Some(1)),
            ("You have 2 minutes left to wait.", Some(120)),
            ("You have 1m 47s left to wait.", Some(107)),
            ("YOU HAVE 3 MINUTES AND 2 SECONDS LEFT TO WAIT.", Some(182)),
            ("Please wait 8 seconds before trying again.", Some(8)),
            ("You have to wait soon.", None),
        ];

        for (message, expected) in cases {
            assert_eq!(extract_wait_time(message), expected, "{message}");
        }
    }
}
