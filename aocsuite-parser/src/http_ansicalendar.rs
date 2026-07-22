use std::collections::HashMap;

use aocsuite_utils::{PuzzleDay, PuzzleId, PuzzleYear};
use regex::Regex;
use scraper::{Html, Selector};

use crate::{ParserError, ParserResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calendar {
    pub rows: Vec<CalendarRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarRow {
    pub cells: Vec<CalendarCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarCell {
    pub text: String,
    pub color: Rgb,
    pub stars: Option<CalendarStars>,
    pub puzzle: Option<PuzzleId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarStars {
    One,
    Two,
}

pub fn parse_calendar(html: &str) -> ParserResult<Calendar> {
    let class_color_map = build_color_map(html);
    let document = Html::parse_document(html);
    let pre_selector = Selector::parse("pre.calendar").expect("valid calendar selector");
    let pre_element = document
        .select(&pre_selector)
        .next()
        .ok_or(ParserError::MissingCalendar)?;
    let rows = process_calendar_content(pre_element, &class_color_map, None, None)?
        .into_iter()
        .map(|cells| CalendarRow { cells })
        .collect();
    Ok(Calendar { rows })
}

fn build_color_map(html: &str) -> HashMap<String, Rgb> {
    let mut class_color_map = HashMap::from([
        ("calendar-day".to_string(), Rgb::new(0xcc, 0xcc, 0xcc)),
        (
            "calendar-mark-complete".to_string(),
            Rgb::new(0xff, 0xff, 0x66),
        ),
        (
            "calendar-mark-verycomplete".to_string(),
            Rgb::new(0xff, 0xff, 0x66),
        ),
    ]);
    let style_re = Regex::new(r"\.calendar-color-([\w\d]+)\s*\{\s*color:\s*(#[0-9a-fA-F]{6});")
        .expect("valid calendar color regex");

    for capture in style_re.captures_iter(html) {
        if let Some(color) = Rgb::from_hex(&capture[2]) {
            class_color_map.insert(format!("calendar-color-{}", &capture[1]), color);
        }
    }
    class_color_map
}

fn process_calendar_content(
    element: scraper::ElementRef,
    class_color_map: &HashMap<String, Rgb>,
    current_stars: Option<CalendarStars>,
    current_puzzle: Option<PuzzleId>,
) -> ParserResult<Vec<Vec<CalendarCell>>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();

    for node in element.children() {
        match node.value() {
            scraper::Node::Text(text) => {
                for (index, line) in text.split('\n').enumerate() {
                    if index > 0 && (!current_row.is_empty() || !rows.is_empty()) {
                        rows.push(current_row);
                        current_row = Vec::new();
                    }
                    if !line.is_empty() {
                        current_row.push(CalendarCell {
                            text: line.to_string(),
                            color: Rgb::DIM,
                            stars: current_stars,
                            puzzle: current_puzzle,
                        });
                    }
                }
            }
            scraper::Node::Element(element) if element.name() == "span" => {
                let class = element.attr("class").unwrap_or_default();
                let content = node
                    .first_child()
                    .and_then(|child| child.value().as_text())
                    .map(|text| text.to_string())
                    .unwrap_or_default();
                if should_skip_marker(class, &content, current_stars) {
                    continue;
                }
                current_row.push(CalendarCell {
                    text: content,
                    color: class_color_map
                        .get(&class.to_owned())
                        .copied()
                        .unwrap_or(Rgb::DEFAULT),
                    stars: current_stars,
                    puzzle: current_puzzle,
                });
            }
            scraper::Node::Element(element) if element.name() == "i" => {
                let content = node
                    .first_child()
                    .and_then(|child| child.value().as_text())
                    .map(|text| text.to_string())
                    .unwrap_or_default();
                current_row.push(CalendarCell {
                    text: content,
                    color: Rgb::DIM,
                    stars: current_stars,
                    puzzle: current_puzzle,
                });
            }
            scraper::Node::Element(element) => {
                let label = element.attr("aria-label").unwrap_or_default();
                let stars = if label.contains("two stars") {
                    Some(CalendarStars::Two)
                } else if label.contains("one star") {
                    Some(CalendarStars::One)
                } else {
                    current_stars
                };
                let puzzle = match element.attr("href") {
                    Some(href) => parse_puzzle_id(href).or(current_puzzle),
                    None => current_puzzle,
                };
                let sub_rows = process_calendar_content(
                    scraper::ElementRef::wrap(node).expect("element node"),
                    class_color_map,
                    stars,
                    puzzle,
                )?;
                for (index, sub_row) in sub_rows.into_iter().enumerate() {
                    if index == 0 {
                        current_row.extend(sub_row);
                    } else {
                        rows.push(current_row);
                        current_row = sub_row;
                    }
                }
            }
            _ => {}
        }
    }

    if !current_row.is_empty() {
        rows.push(current_row);
    }
    Ok(rows)
}

fn parse_puzzle_id(href: &str) -> Option<PuzzleId> {
    let mut segments = href.trim_start_matches('/').split('/');
    let (Some(year), Some("day"), Some(day)) = (segments.next(), segments.next(), segments.next())
    else {
        return None;
    };
    let year = year
        .parse()
        .ok()
        .and_then(|year| PuzzleYear::new(year).ok())?;
    let day = day.parse().ok().and_then(|day| PuzzleDay::new(day).ok())?;
    Some(PuzzleId::new(day, year))
}

fn should_skip_marker(class: &str, content: &str, stars: Option<CalendarStars>) -> bool {
    match stars {
        Some(CalendarStars::Two) => false,
        Some(CalendarStars::One) => class == "calendar-mark-verycomplete" && content.contains('*'),
        None => content.contains('*'),
    }
}

impl Rgb {
    const DIM: Self = Self::new(0x66, 0x66, 0x66);
    const DEFAULT: Self = Self::new(0xcc, 0xcc, 0xcc);

    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    fn from_hex(hex: &str) -> Option<Self> {
        Some(Self::new(
            u8::from_str_radix(&hex[1..3], 16).ok()?,
            u8::from_str_radix(&hex[3..5], 16).ok()?,
            u8::from_str_radix(&hex[5..7], 16).ok()?,
        ))
    }
}
