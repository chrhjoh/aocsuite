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
    let re = Regex::new(
        r"(?i)\byou\s+have(?:\s+to\s+wait)?\s+(?:(\d+)\s*(?:minutes?|mins?|m)\b(?:\s*(?:and\s*)?(\d+)\s*(?:seconds?|secs?|s)\b)?|(\d+)\s*(?:seconds?|secs?|s)\b)",
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
            ("You have to wait soon.", None),
        ];

        for (message, expected) in cases {
            assert_eq!(extract_wait_time(message), expected, "{message}");
        }
    }
}
