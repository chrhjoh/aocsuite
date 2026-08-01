use std::collections::HashMap;

use aocsuite_parser::{Calendar, CalendarStars};
use aocsuite_utils::PuzzleId;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::app::{App, DescriptionState, Tab};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(2),
        ])
        .split(frame.area());
    render_tabs(frame, root[0], app);
    match app.active_tab {
        Tab::Calendar => render_calendar_tab(frame, root[1], app),
        Tab::Language => render_placeholder(
            frame,
            root[1],
            "Language",
            &format!(
                "Language management is the next implementation slice.\nCurrent language: {}",
                app.language
            ),
        ),
        Tab::Config => render_placeholder(
            frame,
            root[1],
            "Config",
            "Configuration management is the next implementation slice.",
        ),
    }
    render_footer(frame, root[2], app);
}

fn render_tabs(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let titles = Tab::ALL
        .iter()
        .map(|tab| Line::from(tab.title()))
        .collect::<Vec<_>>();
    let selected = Tab::ALL
        .iter()
        .position(|tab| *tab == app.active_tab)
        .expect("active tab exists");
    frame.render_widget(
        Tabs::new(titles)
            .select(selected)
            .block(Block::default().borders(Borders::ALL).title(" AoC Suite "))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(" | "),
        area,
    );
}

fn render_calendar_tab(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let direction = if area.width >= 100 {
        Direction::Horizontal
    } else {
        Direction::Vertical
    };
    let constraints = if direction == Direction::Horizontal {
        [Constraint::Percentage(46), Constraint::Percentage(54)]
    } else {
        [Constraint::Percentage(60), Constraint::Percentage(40)]
    };
    let panes = Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area);
    render_calendar(frame, panes[0], app);
    render_description(frame, panes[1], app);
}

fn render_calendar(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let title = if app.calendar_loading {
        format!(" {} - loading... ", app.selected_year)
    } else {
        format!(" {} ", app.selected_year)
    };
    let body = match &app.calendar {
        Some(calendar) => {
            let (stars, total, completed) = completion(calendar);
            let mut lines = vec![Line::from(vec![
                Span::styled(
                    format!("{stars}/{total} stars"),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(format!("  {completed} completed days")),
                Span::raw(format!("  selected day {}", app.selected_day)),
            ])];
            lines.push(Line::default());
            lines.extend(calendar.rows.iter().map(|row| {
                Line::from(
                    row.cells
                        .iter()
                        .map(|cell| {
                            let mut style = Style::default().fg(Color::Rgb(
                                cell.color.red,
                                cell.color.green,
                                cell.color.blue,
                            ));
                            if cell.puzzle == Some(app.selected_puzzle()) {
                                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
                            }
                            Span::styled(cell.text.clone(), style)
                        })
                        .collect::<Vec<_>>(),
                )
            }));
            lines
        }
        None if app.calendar_loading => vec![Line::from("Loading calendar...")],
        None => vec![Line::from("Calendar unavailable. Press r to retry.")],
    };
    frame.render_widget(
        Paragraph::new(body)
            .block(Block::default().borders(Borders::ALL).title(title))
            .scroll(app.calendar_scroll),
        area,
    );
}

fn render_description(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let puzzle = app.selected_puzzle();
    let downloading = app.description_downloading(puzzle);
    let (title, text) = match &app.description {
        DescriptionState::Empty if downloading => (
            format!(" Day {} - downloading... ", puzzle.day),
            "Downloading puzzle description...".to_owned(),
        ),
        DescriptionState::Empty => (
            format!(" Day {} ", puzzle.day),
            "Press d to download this puzzle description.".to_owned(),
        ),
        DescriptionState::Loaded { markdown, .. } => {
            let title = if downloading {
                format!(" Day {} - downloading... ", puzzle.day)
            } else {
                format!(" Day {} ", puzzle.day)
            };
            (title, markdown.clone())
        }
        DescriptionState::Error { .. } if downloading => (
            format!(" Day {} - downloading... ", puzzle.day),
            "Downloading puzzle description...".to_owned(),
        ),
        DescriptionState::Error { message, .. } => {
            (format!(" Day {} - error ", puzzle.day), message.clone())
        }
    };
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false })
            .scroll((app.description_scroll, 0)),
        area,
    );
}

fn render_placeholder(frame: &mut Frame<'_>, area: Rect, title: &str, text: &str) {
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {title} ")),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let help = match app.active_tab {
        Tab::Calendar => {
            "q quit | Tab switch | arrows select | Ctrl+arrows scroll calendar | d download | r refresh | b browser | o open"
        }
        _ => "q quit | Tab/Shift-Tab switch",
    };
    let text = match &app.status {
        Some(status) => format!("{help}\n{status}"),
        None => help.to_owned(),
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Gray)),
        area,
    );
}

fn completion(calendar: &Calendar) -> (usize, usize, usize) {
    let mut stars_by_puzzle = HashMap::<PuzzleId, usize>::new();
    for cell in calendar.rows.iter().flat_map(|row| &row.cells) {
        let Some(puzzle) = cell.puzzle else {
            continue;
        };
        let stars = match cell.stars {
            Some(CalendarStars::One) => 1,
            Some(CalendarStars::Two) => 2,
            None => 0,
        };
        stars_by_puzzle
            .entry(puzzle)
            .and_modify(|current| *current = (*current).max(stars))
            .or_insert(stars);
    }
    let earned = stars_by_puzzle.values().sum();
    let completed = stars_by_puzzle
        .values()
        .filter(|stars| **stars == 2)
        .count();
    (earned, stars_by_puzzle.len() * 2, completed)
}

#[cfg(test)]
mod tests {
    use aocsuite_parser::{Calendar, CalendarCell, CalendarRow, Rgb};
    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzleYear};
    use ratatui::{backend::TestBackend, Terminal};

    use crate::app::{Action, App, Tab};

    use super::render;

    fn app() -> App {
        App::new(
            None,
            PuzzleId::new(PuzzleDay::new(10).unwrap(), PuzzleYear::new(2026).unwrap()),
            LanguageId::Rust,
        )
    }

    #[test]
    fn calendar_tab_renders_in_a_narrow_terminal() {
        let backend = TestBackend::new(70, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = app();

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Calendar"));
        assert!(rendered.contains("Press d to download"));
    }

    #[test]
    fn redownload_keeps_the_existing_preview_visible() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let puzzle = app.selected_puzzle();
        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(Some("existing preview".to_owned())),
        });
        app.update(Action::DownloadDescription);

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("existing preview"));
        assert!(rendered.contains("downloading..."));
        assert!(rendered.contains("d download"));
    }

    #[test]
    fn all_tabs_render() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();

        app.update(Action::NextTab);
        assert_eq!(app.active_tab, Tab::Language);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        assert!(buffer_text(terminal.backend().buffer()).contains("next implementation slice"));

        app.update(Action::NextTab);
        assert_eq!(app.active_tab, Tab::Config);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        assert!(buffer_text(terminal.backend().buffer()).contains("Configuration management"));
    }

    #[test]
    fn narrow_calendar_can_scroll_to_clipped_rows() {
        let backend = TestBackend::new(70, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.calendar = Some(Calendar {
            rows: (0..15)
                .map(|row| CalendarRow {
                    cells: vec![CalendarCell {
                        text: format!("calendar-row-{row:02}"),
                        color: Rgb {
                            red: 255,
                            green: 255,
                            blue: 255,
                        },
                        stars: None,
                        puzzle: None,
                    }],
                })
                .collect(),
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();
        assert!(!buffer_text(terminal.backend().buffer()).contains("calendar-row-14"));

        for _ in 0..8 {
            app.update(Action::ScrollCalendarDown);
        }
        assert_eq!(app.calendar_scroll.0, 8);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("calendar-row-14"), "{rendered}");
    }

    fn buffer_text(buffer: &ratatui::buffer::Buffer) -> String {
        let area = buffer.area;
        (area.y..area.y + area.height)
            .map(|y| {
                (area.x..area.x + area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
