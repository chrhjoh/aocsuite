use colored::*;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub trait HttpParser {
    fn parse(&self, html: &str) -> String;
}

pub struct AnsiCalandarParser;
pub struct ArticleMarkdownParser;
pub struct DummyParser;

impl HttpParser for AnsiCalandarParser {
    fn parse(&self, html: &str) -> String {
        let class_color_map = build_color_map(html);

        let document = Html::parse_document(html);
        let pre_selector = Selector::parse("pre.calendar").unwrap();

        for pre_element in document.select(&pre_selector) {
            let rows = process_calendar_content(pre_element, &class_color_map, None);
            return rows
                .into_iter()
                .map(|row| row.join(""))
                .collect::<Vec<String>>()
                .join("\n");
        }

        String::new()
    }
}
fn build_color_map(html: &str) -> HashMap<String, String> {
    let mut class_color_map = HashMap::from([
        ("calendar-day".to_string(), "#cccccc".to_string()),
        ("calendar-mark-complete".to_string(), "#ffff66".to_string()),
        (
            "calendar-mark-verycomplete".to_string(),
            "#ffff66".to_string(),
        ),
    ]);
    let style_re =
        Regex::new(r"\.calendar-color-([\w\d]+)\s*\{\s*color:\s*(#[0-9a-fA-F]{6});").unwrap();

    for cap in style_re.captures_iter(html) {
        let class = format!("calendar-color-{}", &cap[1]);
        class_color_map.insert(class, cap[2].to_string());
    }
    class_color_map
}

fn process_calendar_content(
    element: scraper::ElementRef,
    class_color_map: &HashMap<String, String>,
    current_stars: Option<CalendarStars>,
) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();

    for node in element.children() {
        match node.value() {
            scraper::Node::Text(text) => {
                let lines: Vec<&str> = text.split('\n').collect();
                for (i, line) in lines.iter().enumerate() {
                    if i > 0 {
                        if !current_row.is_empty() || !rows.is_empty() {
                            rows.push(current_row);
                            current_row = Vec::new();
                        }
                    }
                    if !line.is_empty() {
                        let hex = "#666666";

                        let colored_content = if let Some(rgb) = parse_hex_color(hex) {
                            line.truecolor(rgb.0, rgb.1, rgb.2).to_string()
                        } else {
                            line.to_string()
                        };
                        current_row.push(colored_content);
                    }
                }
            }
            scraper::Node::Element(elem) => {
                if elem.name() == "span" {
                    let class = elem.attr("class").unwrap_or_default();
                    match &current_stars {
                        Some(CalendarStars::Two) => {}
                        Some(CalendarStars::One) => {
                            if class == "calendar-mark-verycomplete" {
                                continue;
                            }
                        }
                        None => continue,
                    }
                    let content = node
                        .first_child()
                        .and_then(|child| child.value().as_text())
                        .map(|text| text.to_string())
                        .unwrap_or_default();

                    let hex = match class_color_map.get(class) {
                        Some(val) => val,
                        None => "#cccccc",
                    };

                    let colored_content = if let Some(rgb) = parse_hex_color(hex) {
                        content.truecolor(rgb.0, rgb.1, rgb.2).to_string()
                    } else {
                        content
                    };

                    current_row.push(colored_content);
                } else if elem.name() == "i" {
                    let content = node
                        .first_child()
                        .and_then(|child| child.value().as_text())
                        .map(|text| text.to_string())
                        .unwrap_or_default();
                    let hex = "#666666";

                    let colored_content = if let Some(rgb) = parse_hex_color(hex) {
                        content.truecolor(rgb.0, rgb.1, rgb.2).to_string()
                    } else {
                        content
                    };
                    current_row.push(colored_content);
                } else {
                    let label = elem.attr("aria-label").unwrap_or_default();
                    let current_stars = if label.contains("two stars") {
                        Some(CalendarStars::Two)
                    } else if label.contains("one star") {
                        Some(CalendarStars::One)
                    } else {
                        None
                    };
                    let sub_rows = process_calendar_content(
                        scraper::ElementRef::wrap(node).unwrap(),
                        class_color_map,
                        current_stars,
                    );

                    for (i, sub_row) in sub_rows.into_iter().enumerate() {
                        if i == 0 {
                            current_row.extend(sub_row);
                        } else {
                            rows.push(current_row);
                            current_row = sub_row;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Don't forget the last row
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    rows
}

#[derive(PartialEq, Eq)]
enum CalendarStars {
    One,
    Two,
}

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

impl HttpParser for DummyParser {
    fn parse(&self, html: &str) -> String {
        html.to_string()
    }
}

// Helper: convert "#RRGGBB" to (r, g, b)
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    if hex.len() == 7 && hex.starts_with('#') {
        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ParserType {
    Colored,
    Markdown,
    Raw,
}

fn get_parser(parser_type: ParserType) -> Box<dyn HttpParser> {
    match parser_type {
        ParserType::Colored => Box::new(AnsiCalandarParser),
        ParserType::Markdown => Box::new(ArticleMarkdownParser),
        ParserType::Raw => Box::new(DummyParser),
    }
}

pub fn parse_html(html: &str, parser_type: ParserType) -> String {
    let parser = get_parser(parser_type);
    parser.parse(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_selection() {
        let html = r#"<div><strong>Bold text</strong></div>"#;

        let markdown_result = parse_html(html, ParserType::Markdown);
        let raw_result = parse_html(html, ParserType::Raw);

        assert_eq!(raw_result, html);
        assert!(markdown_result.contains("**Bold text**"));
    }

    #[test]
    fn test_markdown_parser() {
        let html = r#"<div><strong>Bold text</strong> and <em>italic text</em></div>"#;
        let parser = ArticleMarkdownParser;
        let result = parser.parse(html);
        assert!(result.contains("**Bold text**"));
        assert!(result.contains("*italic text*"));
    }

    #[test]
    fn test_raw_html_parser() {
        let html = r#"<div>Test content</div>"#;
        let parser = DummyParser;
        let result = parser.parse(html);
        assert_eq!(result, html);
    }

    #[test]
    fn test_colored_html_parser() {
        let html = r#"
    <style>
    .calendar .calendar-color-g0 { color:#488813; }
    .calendar .calendar-color-g1 { color:#4d8b03; }
    .calendar .calendar-color-c { color:#eeeeee; }
    </style>
    <pre class="calendar">
        <span class="calendar-color-g0">Item A</span>
        <span class="calendar-color-g1">Item B</span>
        <span class="calendar-color-c">Item C</span>
    </pre>
    "#;

        let parser = AnsiCalandarParser;
        let result = parser.parse(html);
        assert!(result.contains("Item A"));
        assert!(result.contains("Item B"));
        assert!(result.contains("Item C"));
    }
}
