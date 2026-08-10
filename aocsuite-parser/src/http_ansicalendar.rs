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

struct ColorRule {
    selector: Selector,
    value: ColorValue,
    important: bool,
    specificity: (usize, usize, usize),
    source_order: usize,
}

#[derive(Clone, Copy)]
enum ColorValue {
    Rgb(Rgb),
    Inherit,
}

#[derive(Clone, Copy)]
struct ColorDeclaration {
    value: ColorValue,
    important: bool,
}

pub fn parse_calendar(html: &str) -> ParserResult<Calendar> {
    let document = Html::parse_document(html);
    let color_rules = build_color_rules(&document);
    let pre_selector = Selector::parse("pre.calendar").expect("valid calendar selector");
    let pre_element = document
        .select(&pre_selector)
        .next()
        .ok_or(ParserError::MissingCalendar)?;
    let root_color = resolve_color(pre_element, &color_rules, Rgb::DIM);
    let rows = process_calendar_content(pre_element, &color_rules, None, None, root_color)?
        .into_iter()
        .map(|cells| CalendarRow { cells })
        .collect();
    Ok(Calendar { rows })
}

fn build_color_rules(document: &Html) -> Vec<ColorRule> {
    let mut color_rules = Vec::new();
    let style_selector = Selector::parse("style").expect("valid style selector");
    let rule_re = Regex::new(r"(?s)([^{}]+)\{([^{}]*)\}").expect("valid CSS rule regex");
    let color_re = Regex::new(
        r"(?i)(?:^|;)\s*color\s*:\s*(inherit|#[0-9a-f]{3}(?:[0-9a-f]{3})?)\s*(!important)?\s*(?:;|$)",
    )
    .expect("valid CSS color regex");

    for style in document.select(&style_selector) {
        let stylesheet = style.text().collect::<String>();
        for rule in rule_re.captures_iter(&stylesheet) {
            let Some((_, color)) = color_re
                .captures_iter(&rule[2])
                .enumerate()
                .max_by_key(|(index, color)| (color.get(2).is_some(), *index))
            else {
                continue;
            };
            let value = if color[1].eq_ignore_ascii_case("inherit") {
                ColorValue::Inherit
            } else if let Some(color) = Rgb::from_hex(&color[1]) {
                ColorValue::Rgb(color)
            } else {
                continue;
            };
            let important = color.get(2).is_some();
            for selector_text in rule[1].split(',').map(str::trim) {
                if selector_text.is_empty()
                    || selector_text.contains(':')
                    || selector_text.contains('[')
                {
                    continue;
                }
                let Ok(selector) = Selector::parse(selector_text) else {
                    continue;
                };
                color_rules.push(ColorRule {
                    selector,
                    value,
                    important,
                    specificity: selector_specificity(selector_text),
                    source_order: color_rules.len(),
                });
            }
        }
    }
    color_rules
}

fn selector_specificity(selector: &str) -> (usize, usize, usize) {
    let ids = selector.matches('#').count();
    let classes = selector.matches('.').count();
    let types = selector
        .split(|character: char| character.is_ascii_whitespace() || ">+~".contains(character))
        .filter(|compound| {
            let type_selector = compound.split(['.', '#']).next().unwrap_or_default();
            !type_selector.is_empty() && type_selector != "*"
        })
        .count();
    (ids, classes, types)
}

fn resolve_color(element: scraper::ElementRef, color_rules: &[ColorRule], inherited: Rgb) -> Rgb {
    let stylesheet = color_rules
        .iter()
        .filter(|rule| rule.selector.matches(&element))
        .max_by_key(|rule| (rule.important, rule.specificity, rule.source_order))
        .map(|rule| ColorDeclaration {
            value: rule.value,
            important: rule.important,
        });
    let inline = element.value().attr("style").and_then(parse_inline_color);
    let specified = match (stylesheet, inline) {
        (Some(stylesheet), Some(inline)) if stylesheet.important && !inline.important => stylesheet,
        (_, Some(inline)) => inline,
        (Some(stylesheet), None) => stylesheet,
        (None, None) => {
            return fallback_color(element.value().attr("class").unwrap_or_default())
                .unwrap_or(inherited);
        }
    };
    match specified.value {
        ColorValue::Rgb(color) => color,
        ColorValue::Inherit => inherited,
    }
}

fn parse_inline_color(style: &str) -> Option<ColorDeclaration> {
    style
        .split(';')
        .filter_map(|declaration| {
            let (property, value) = declaration.split_once(':')?;
            if !property.trim().eq_ignore_ascii_case("color") {
                return None;
            }
            let value = value.trim();
            let (value, important) = match value.rsplit_once('!') {
                Some((value, priority)) if priority.trim().eq_ignore_ascii_case("important") => {
                    (value.trim(), true)
                }
                _ => (value, false),
            };
            let value = if value.eq_ignore_ascii_case("inherit") {
                ColorValue::Inherit
            } else {
                ColorValue::Rgb(Rgb::from_hex(value)?)
            };
            Some(ColorDeclaration { value, important })
        })
        .fold(None, |current, declaration| match current {
            Some(current) if current.important && !declaration.important => Some(current),
            _ => Some(declaration),
        })
}

fn fallback_color(classes: &str) -> Option<Rgb> {
    if has_class(classes, "calendar-day") {
        Some(Rgb::new(0xcc, 0xcc, 0xcc))
    } else if has_class(classes, "calendar-mark-complete")
        || has_class(classes, "calendar-mark-verycomplete")
    {
        Some(Rgb::new(0xff, 0xff, 0x66))
    } else {
        None
    }
}

fn process_calendar_content(
    element: scraper::ElementRef,
    color_rules: &[ColorRule],
    current_stars: Option<CalendarStars>,
    current_puzzle: Option<PuzzleId>,
    current_color: Rgb,
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
                            color: current_color,
                            stars: current_stars,
                            puzzle: current_puzzle,
                        });
                    }
                }
            }
            scraper::Node::Element(element) => {
                if element.attr("style").is_some_and(is_absolutely_positioned) {
                    continue;
                }
                let child_element = scraper::ElementRef::wrap(node).expect("element node");
                let classes = element.attr("class").unwrap_or_default();
                if should_skip_marker(classes, current_stars) {
                    continue;
                }
                let label = element.attr("aria-label").unwrap_or_default();
                let parsed_puzzle = element.attr("href").and_then(parse_puzzle_id);
                let stars = if parsed_puzzle.is_some() {
                    stars_from_link(classes, label)
                } else {
                    current_stars
                };
                let puzzle = parsed_puzzle.or(current_puzzle);
                let color = resolve_color(child_element, color_rules, current_color);
                let sub_rows =
                    process_calendar_content(child_element, color_rules, stars, puzzle, color)?;
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

fn stars_from_link(classes: &str, label: &str) -> Option<CalendarStars> {
    if has_class(classes, "calendar-verycomplete") {
        Some(CalendarStars::Two)
    } else if has_class(classes, "calendar-complete") {
        Some(CalendarStars::One)
    } else if label.contains("two stars") {
        Some(CalendarStars::Two)
    } else if label.contains("one star") {
        Some(CalendarStars::One)
    } else {
        None
    }
}

fn should_skip_marker(classes: &str, stars: Option<CalendarStars>) -> bool {
    if has_class(classes, "calendar-mark-verycomplete") {
        stars != Some(CalendarStars::Two)
    } else if has_class(classes, "calendar-mark-complete") {
        stars.is_none()
    } else {
        false
    }
}

fn has_class(classes: &str, expected: &str) -> bool {
    classes
        .split_ascii_whitespace()
        .any(|class| class == expected)
}

fn is_absolutely_positioned(style: &str) -> bool {
    style.split(';').any(|declaration| {
        let Some((property, value)) = declaration.split_once(':') else {
            return false;
        };
        property.trim().eq_ignore_ascii_case("position")
            && value
                .split('!')
                .next()
                .is_some_and(|value| value.trim().eq_ignore_ascii_case("absolute"))
    })
}

impl Rgb {
    const DIM: Self = Self::new(0x66, 0x66, 0x66);

    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    fn from_hex(hex: &str) -> Option<Self> {
        match hex.len() {
            4 => Some(Self::new(
                u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?,
            )),
            7 => Some(Self::new(
                u8::from_str_radix(&hex[1..3], 16).ok()?,
                u8::from_str_radix(&hex[3..5], 16).ok()?,
                u8::from_str_radix(&hex[5..7], 16).ok()?,
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use aocsuite_utils::{PuzzleDay, PuzzleId, PuzzleYear};

    use super::{CalendarStars, Rgb, parse_calendar};

    #[test]
    fn parses_nested_2017_calendar_content_with_colors_and_completion() {
        let calendar = parse_calendar(
            r#"
                <style>
                    .calendar-edge { color: #123; }
                    .calendar-disabled { color: #234567; }
                    .calendar-ornament3 { color: #abcdef; }
                </style>
                <pre class="calendar"><a aria-label="Day 1" href="/2017/day/1" class="calendar-day1 calendar-verycomplete"><span class="calendar-edge"><i>|</i></span><span class="calendar-disabled"><i>A</i><span class="calendar-ornament3"><i>*</i></span><i>B</i></span> <span class="calendar-day"> 1</span> <span class="calendar-mark-complete">*</span><span class="calendar-mark-verycomplete">*</span></a></pre>
            "#,
        )
        .unwrap();

        assert_eq!(calendar.rows.len(), 1);
        assert_eq!(row_text(&calendar.rows[0].cells), "|A*B  1 **");

        let puzzle = PuzzleId::new(PuzzleDay::new(1).unwrap(), PuzzleYear::new(2017).unwrap());
        assert!(
            calendar.rows[0].cells.iter().all(|cell| {
                cell.puzzle == Some(puzzle) && cell.stars == Some(CalendarStars::Two)
            })
        );
        assert!(calendar.rows[0].cells.iter().any(|cell| {
            cell.text == "|"
                && cell.color
                    == Rgb {
                        red: 0x11,
                        green: 0x22,
                        blue: 0x33,
                    }
        }));
        assert!(calendar.rows[0].cells.iter().any(|cell| {
            cell.text == "*"
                && cell.color
                    == Rgb {
                        red: 0xab,
                        green: 0xcd,
                        blue: 0xef,
                    }
        }));
    }

    #[test]
    fn parses_2015_ornaments_and_multiline_trunk() {
        let calendar = parse_calendar(
            r#"<pre class="calendar"><a aria-label="Day 1" href="/2015/day/1" class="calendar-day1 calendar-complete">A<span class="calendar-ornament3">*</span>B <span class="calendar-day"> 1</span> <span class="calendar-mark-complete">*</span><span class="calendar-mark-verycomplete">*</span></a>
<span class="calendar-trunk">x
y</span></pre>"#,
        )
        .unwrap();

        let rows = calendar
            .rows
            .iter()
            .map(|row| row_text(&row.cells))
            .collect::<Vec<_>>();
        assert_eq!(rows, ["A*B  1 *", "x", "y"]);
        assert!(
            calendar.rows[0]
                .cells
                .iter()
                .all(|cell| cell.stars == Some(CalendarStars::One))
        );
    }

    #[test]
    fn applies_2015_ornament_colors_only_in_their_completion_context() {
        let calendar = parse_calendar(
            r#"<style>
.calendar a.calendar-verycomplete .calendar-ornament0 { color: #0066ff; text-shadow: 0 0 5px #0066ff; }
.calendar a.calendar-verycomplete .calendar-ornament1 { color: #ff9900; text-shadow: 0 0 5px #ff9900; }
.calendar a.calendar-verycomplete .calendar-ornament2 { color: #ff0000; text-shadow: 0 0 5px #ff0000; }
.calendar a.calendar-verycomplete .calendar-ornament3 { color: #ffff66; text-shadow: 0 0 5px #ffff66; }
.calendar a .calendar-ornament0 { color: inherit; }
.calendar a .calendar-ornament1 { color: inherit; }
.calendar a .calendar-ornament2 { color: inherit; }
.calendar a .calendar-ornament3 { color: inherit; }
</style>
<pre class="calendar"><a aria-label="Day 6" href="/2015/day/6" class="calendar-day6">x<span class="calendar-ornament0">O</span></a>
<a aria-label="Day 7" href="/2015/day/7" class="calendar-day7 calendar-verycomplete">v<span class="calendar-ornament0">O</span><span class="calendar-ornament1">o</span><span class="calendar-ornament2">@</span><span class="calendar-ornament3">*</span></a></pre>"#,
        )
        .unwrap();

        assert_eq!(
            row_cells(&calendar.rows[0].cells),
            [("x", Rgb::DIM), ("O", Rgb::DIM)]
        );
        assert_eq!(
            row_cells(&calendar.rows[1].cells),
            [
                ("v", Rgb::DIM),
                (
                    "O",
                    Rgb {
                        red: 0x00,
                        green: 0x66,
                        blue: 0xff,
                    },
                ),
                (
                    "o",
                    Rgb {
                        red: 0xff,
                        green: 0x99,
                        blue: 0x00,
                    },
                ),
                (
                    "@",
                    Rgb {
                        red: 0xff,
                        green: 0x00,
                        blue: 0x00,
                    },
                ),
                (
                    "*",
                    Rgb {
                        red: 0xff,
                        green: 0xff,
                        blue: 0x66,
                    },
                ),
            ]
        );
    }

    #[test]
    fn placeholder_markers_do_not_imply_completion_or_hide_ornaments() {
        let calendar = parse_calendar(
            r#"<pre class="calendar"><a aria-label="Day 1" href="/2015/day/1" class="calendar-day1"><span class="calendar-ornament3">*</span><span class="calendar-mark-complete">*</span><span class="calendar-mark-verycomplete">*</span></a></pre>"#,
        )
        .unwrap();

        assert_eq!(row_text(&calendar.rows[0].cells), "*");
        assert!(
            calendar.rows[0]
                .cells
                .iter()
                .all(|cell| cell.stars.is_none())
        );
    }

    #[test]
    fn aria_label_remains_a_completion_fallback() {
        let calendar = parse_calendar(
            r#"<pre class="calendar"><a aria-label="Day 1, one star" href="/2025/day/1" class="calendar-day1"><span class="calendar-mark-complete">*</span><span class="calendar-mark-verycomplete">*</span></a></pre>"#,
        )
        .unwrap();

        assert_eq!(row_text(&calendar.rows[0].cells), "*");
        assert!(
            calendar.rows[0]
                .cells
                .iter()
                .all(|cell| cell.stars == Some(CalendarStars::One))
        );
    }

    #[test]
    fn omits_2024_absolute_positioned_overlay_glyphs() {
        let calendar = parse_calendar(
            r#"<pre class="calendar"><a aria-label="Day 14, two stars" href="/2024/day/14" class="calendar-day14 calendar-verycomplete"><span class="calendar-color-9c">.<span style="color:#5296e6; position:absolute; left:0.00em; opacity:0.30;">.</span><span style="POSITION: absolute !important; left:0.05em; opacity:0.27;">.</span></span><span style="position:relative; color:#123">:</span><span class="calendar-day">14</span><span class="calendar-mark-complete">*</span><span class="calendar-mark-verycomplete">*</span></a></pre>"#,
        )
        .unwrap();

        assert_eq!(row_text(&calendar.rows[0].cells), ".:14**");
        assert!(
            calendar.rows[0]
                .cells
                .iter()
                .all(|cell| cell.stars == Some(CalendarStars::Two))
        );
        assert!(calendar.rows[0].cells.iter().any(|cell| {
            cell.text == ":"
                && cell.color
                    == Rgb {
                        red: 0x11,
                        green: 0x22,
                        blue: 0x33,
                    }
        }));
    }

    fn row_text(cells: &[super::CalendarCell]) -> String {
        cells.iter().map(|cell| cell.text.as_str()).collect()
    }

    fn row_cells(cells: &[super::CalendarCell]) -> Vec<(&str, Rgb)> {
        cells
            .iter()
            .map(|cell| (cell.text.as_str(), cell.color))
            .collect()
    }
}
