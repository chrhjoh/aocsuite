use scraper::{Html, Selector};

use crate::{ParserError, ParserResult};

pub fn parse_puzzle_markdown(html: &str) -> ParserResult<String> {
    let markdown = parse_article_markdown(html);
    if markdown.trim().is_empty() {
        Err(ParserError::MissingPuzzleArticle)
    } else {
        Ok(markdown)
    }
}

pub(crate) fn parse_article_markdown(html: &str) -> String {
    let document = Html::parse_document(html);
    let main_selector = Selector::parse("main").expect("valid main selector");
    let article_selector = Selector::parse("article").expect("valid article selector");

    let mut articles_html = String::new();
    for main in document.select(&main_selector) {
        for article in main.select(&article_selector) {
            articles_html.push_str(&article.html());
        }
    }

    html2md::parse_html(&articles_html)
}
