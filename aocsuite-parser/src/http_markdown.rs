use scraper::{Html, Selector};

use crate::HttpParser;

pub struct ArticleMarkdownParser;

impl HttpParser for ArticleMarkdownParser {
    fn parse(&self, html: &str) -> String {
        let document = Html::parse_document(html);

        let main_selector = Selector::parse("main").unwrap();
        let article_selector = Selector::parse("article").unwrap();

        let mut articles_html = String::new();
        for main in document.select(&main_selector) {
            for article in main.select(&article_selector) {
                let html_fragment = article.html(); // includes outer tag
                articles_html.push_str(&html_fragment);
            }
        }

        html2md::parse_html(&articles_html)
    }
}
