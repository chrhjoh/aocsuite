mod http_ansicalendar;
mod http_markdown;
mod http_submission;
use http_ansicalendar::AnsiCalendarParser;
use http_markdown::ArticleMarkdownParser;
pub use http_submission::{AocSubmissionResult, parse_submission_result};

trait HttpParser {
    fn parse(&self, html: &str) -> String;
}

#[derive(Debug, Clone, Copy)]
pub enum ParserType {
    Colored,
    MarkdownArticle,
}

pub fn parse(html: &str, parser_type: ParserType) -> String {
    let parser: Box<dyn HttpParser> = match parser_type {
        ParserType::Colored => Box::new(AnsiCalendarParser),
        ParserType::MarkdownArticle => Box::new(ArticleMarkdownParser),
    };

    parser.parse(html)
}
