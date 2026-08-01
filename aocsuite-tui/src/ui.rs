use std::collections::HashMap;

use aocsuite_parser::{Calendar, CalendarStars};
use aocsuite_utils::PuzzleId;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Tabs,
        Wrap,
    },
    Frame,
};

use crate::app::{
    App, DescriptionState, LanguageConfirmation, LanguageDialog, LanguageFocus,
    LanguageOperationState, LanguageTextInput, Tab,
};

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
        Tab::Language => render_language_tab(frame, root[1], app),
        Tab::Config => render_placeholder(
            frame,
            root[1],
            "Config",
            "Configuration management is the next implementation slice.",
        ),
    }
    render_footer(frame, root[2], app);
    if let Some(dialog) = &app.language_dialog {
        render_language_dialog(frame, dialog);
    }
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
    let title = format!(" {} ", app.selected_year);
    let body = match &app.calendar {
        Some(calendar) => {
            let (stars, total, completed) = completion(calendar);
            let selected_puzzle = app.selected_puzzle();
            let selected_row = selected_puzzle.and_then(|puzzle| {
                calendar
                    .rows
                    .iter()
                    .rposition(|row| row.cells.iter().any(|cell| cell.puzzle == Some(puzzle)))
            });
            let mut lines = vec![Line::from(vec![
                Span::styled(
                    format!("{stars}/{total} stars"),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(format!("  {completed} completed days")),
            ])];
            lines.push(Line::default());
            lines.extend(calendar.rows.iter().enumerate().map(|(row_index, row)| {
                Line::from(
                    row.cells
                        .iter()
                        .map(|cell| {
                            let mut style = Style::default().fg(Color::Rgb(
                                cell.color.red,
                                cell.color.green,
                                cell.color.blue,
                            ));
                            if selected_row == Some(row_index) && cell.puzzle == selected_puzzle {
                                style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
                            }
                            Span::styled(cell.text.clone(), style)
                        })
                        .collect::<Vec<_>>(),
                )
            }));
            lines
        }
        None if app.calendar_loading => Vec::new(),
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
    let Some(puzzle) = app.selected_puzzle() else {
        frame.render_widget(
            Paragraph::new("")
                .block(Block::default().borders(Borders::ALL).title(" Puzzle "))
                .wrap(Wrap { trim: false }),
            area,
        );
        return;
    };
    let downloading = app.description_downloading(puzzle);
    let scrollable = matches!(app.description, DescriptionState::Loaded { .. });
    let (title, text) = match &app.description {
        DescriptionState::CheckingCache(_) => (format!(" Day {} ", puzzle.day), String::new()),
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
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    let content_length = paragraph.line_count(inner.width);
    frame.render_widget(
        paragraph.block(block).scroll((app.description_scroll, 0)),
        area,
    );
    if scrollable && content_length > usize::from(inner.height) {
        let mut state = ScrollbarState::new(content_length)
            .viewport_content_length(usize::from(inner.height))
            .position(usize::from(app.description_scroll));
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(None)
                .thumb_symbol("▐"),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut state,
        );
    }
}

fn render_language_tab(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(4)])
        .split(area);
    let rust_style = if app.language == aocsuite_utils::LanguageId::Rust {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let python_style = if app.language == aocsuite_utils::LanguageId::Python {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Rust", rust_style),
            Span::raw(" | "),
            Span::styled("Python", python_style),
            Span::styled(
                "   s switches for this session",
                Style::default().fg(Color::Gray),
            ),
        ]))
        .block(Block::default().borders(Borders::ALL).title(" Language ")),
        sections[0],
    );

    let direction = if area.width >= 80 {
        Direction::Horizontal
    } else {
        Direction::Vertical
    };
    let panes = Layout::default()
        .direction(direction)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(sections[1]);
    let (package_activity, library_activity) = match &app.language_operation {
        LanguageOperationState::Idle => (None, None),
        LanguageOperationState::Running {
            packages,
            libraries,
        } => (packages.as_deref(), libraries.as_deref()),
    };
    render_language_list(
        frame,
        panes[0],
        "Packages",
        &app.language_packages,
        app.language_package_selection,
        app.language_focus == LanguageFocus::Packages,
        package_activity,
    );
    render_language_list(
        frame,
        panes[1],
        "Libraries",
        &app.language_libraries,
        app.language_library_selection,
        app.language_focus == LanguageFocus::Libraries,
        library_activity,
    );
}

fn render_language_list(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    items: &[String],
    selected: usize,
    focused: bool,
    activity: Option<&str>,
) {
    let title_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let lines = if items.is_empty() {
        vec![Line::styled("None", Style::default().fg(Color::DarkGray))]
    } else {
        items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let is_selected = index == selected;
                let style = if focused && is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Line::styled(
                    format!("{} {item}", if is_selected { ">" } else { " " }),
                    style,
                )
            })
            .collect()
    };
    let visible_rows = usize::from(area.height.saturating_sub(2));
    let offset = selected
        .saturating_add(1)
        .saturating_sub(visible_rows)
        .min(u16::MAX as usize) as u16;
    frame.render_widget(
        Paragraph::new(lines).scroll((offset, 0)).block(
            Block::default().borders(Borders::ALL).title(Span::styled(
                activity.map_or_else(
                    || format!(" {title} "),
                    |activity| format!(" {title} - {activity} "),
                ),
                title_style,
            )),
        ),
        area,
    );
}

fn render_language_dialog(frame: &mut Frame<'_>, dialog: &LanguageDialog) {
    let area = centered_dialog(frame.area());
    if area.width < 4 || area.height < 4 {
        return;
    }
    frame.render_widget(Clear, area);
    match dialog {
        LanguageDialog::Text { kind, value, error } => {
            let (title, prompt) = match kind {
                LanguageTextInput::AddPackage => (" Add package ", "Package name"),
                LanguageTextInput::Library => (" New library ", "Library name"),
            };
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(prompt),
                    Line::styled(value, Style::default().fg(Color::Yellow)),
                    Line::styled(
                        error.as_deref().unwrap_or_default(),
                        Style::default().fg(Color::Red),
                    ),
                    Line::styled(
                        "Enter accept | Esc cancel",
                        Style::default().fg(Color::Gray),
                    ),
                ])
                .block(Block::default().borders(Borders::ALL).title(title)),
                area,
            );
        }
        LanguageDialog::Confirm { action, confirmed } => {
            let (title, question, destructive) = match action {
                LanguageConfirmation::RemovePackage(package) => (
                    " Remove package ",
                    format!("Remove package {package}?"),
                    "Remove",
                ),
                LanguageConfirmation::RemoveLibrary(library) => (
                    " Delete library ",
                    format!("Delete library {library}?"),
                    "Delete",
                ),
                LanguageConfirmation::ResetTemplate => (
                    " Reset template ",
                    "Replace the current template?".to_owned(),
                    "Reset",
                ),
            };
            let cancel_style = if *confirmed {
                Style::default()
            } else {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            };
            let confirm_style = if *confirmed {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(question),
                    Line::default(),
                    Line::from(vec![
                        Span::styled("[ Cancel ]", cancel_style),
                        Span::raw("   "),
                        Span::styled(format!("[ {destructive} ]"), confirm_style),
                    ]),
                    Line::styled(
                        "Left/Right choose | Enter accept | Esc cancel",
                        Style::default().fg(Color::Gray),
                    ),
                ])
                .block(Block::default().borders(Borders::ALL).title(title)),
                area,
            );
        }
        LanguageDialog::Message(message) => {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(message.as_str()),
                    Line::default(),
                    Line::styled("Enter/Esc dismiss", Style::default().fg(Color::Gray)),
                ])
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).title(" Error ")),
                area,
            );
        }
    }
}

fn centered_dialog(area: Rect) -> Rect {
    let width = area.width.saturating_sub(4).min(64);
    let height = area.height.saturating_sub(2).min(7);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
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
    let (help, secondary) = match app.active_tab {
        Tab::Calendar => (
            "q quit | Tab tabs | arrows select | PgUp/PgDn description",
            Some("Ctrl+arrows pan | d download | r refresh | b browser | o open"),
        ),
        Tab::Language => (
            "q quit | Tab tabs | s language | arrows select | r refresh",
            Some("a add package | x remove | n new library | o/Enter open | t/T template"),
        ),
        Tab::Config => ("q quit | Tab/Shift-Tab switch", None),
    };
    let text = match &app.status {
        Some(status) => format!("{help}\n{status}"),
        None => secondary.map_or_else(
            || help.to_owned(),
            |secondary| format!("{help}\n{secondary}"),
        ),
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
    use ratatui::{backend::TestBackend, style::Color, Terminal};

    use crate::app::{Action, App, LanguageData, Tab};

    use super::render;

    fn app() -> App {
        let puzzle = PuzzleId::new(PuzzleDay::new(10).unwrap(), PuzzleYear::new(2026).unwrap());
        let mut app = App::new(None, puzzle, LanguageId::Rust);
        app.update(Action::CalendarFinished {
            year: puzzle.year,
            refresh: false,
            result: Ok(Calendar {
                rows: vec![CalendarRow {
                    cells: vec![CalendarCell {
                        text: "calendar puzzle".to_owned(),
                        color: Rgb {
                            red: 255,
                            green: 255,
                            blue: 255,
                        },
                        stars: None,
                        puzzle: Some(puzzle),
                    }],
                }],
            }),
        });
        app
    }

    #[test]
    fn calendar_tab_renders_in_a_narrow_terminal() {
        let backend = TestBackend::new(70, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = app();
        let selected = app.selected_puzzle().unwrap();

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Calendar"));
        assert!(rendered.contains("Day 10"));
        assert!(!rendered.contains("Press d to download"));
        assert!(!rendered.contains("loading..."));
        assert!(rendered.contains("0/2 stars"));
        assert!(!rendered.contains(&selected.to_string()));
        assert!(rendered.contains("PgUp/PgDn description"));
        assert!(rendered.contains("d download"));
    }

    #[test]
    fn pending_calendar_and_selection_render_blank_panes() {
        let backend = TestBackend::new(70, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let latest = PuzzleId::new(PuzzleDay::new(10).unwrap(), PuzzleYear::new(2026).unwrap());
        let mut app = App::new(None, latest, LanguageId::Rust);
        app.initial_effects();

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("2026"));
        assert!(rendered.contains("Puzzle"));
        assert!(!rendered.to_lowercase().contains("loading"));
        assert!(!rendered.contains("No calendar puzzle is selected"));
    }

    #[test]
    fn cache_check_remains_blank_when_a_download_is_requested() {
        let backend = TestBackend::new(70, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::DownloadDescription);

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Day 10"));
        assert!(!rendered.contains("Press d to download"));
        assert!(!rendered.contains("downloading..."));
    }

    #[test]
    fn confirmed_cache_miss_renders_the_download_prompt() {
        let backend = TestBackend::new(70, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let puzzle = app.selected_puzzle().unwrap();
        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(None),
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();

        assert!(buffer_text(terminal.backend().buffer()).contains("Press d to download"));
    }

    #[test]
    fn redownload_keeps_the_existing_preview_visible() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let puzzle = app.selected_puzzle().unwrap();
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
    fn overflowing_description_renders_a_positioned_scrollbar() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let puzzle = app.selected_puzzle().unwrap();
        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(Some("wrapped description ".repeat(500))),
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();
        let initial_thumb = symbol_y(terminal.backend().buffer(), "▐").unwrap();
        assert!(buffer_text(terminal.backend().buffer()).contains("PgUp/PgDn description"));

        app.description_scroll = 30;
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let scrolled_thumb = symbol_y(terminal.backend().buffer(), "▐").unwrap();

        assert!(scrolled_thumb > initial_thumb);
    }

    #[test]
    fn fitting_description_does_not_render_a_scrollbar() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let puzzle = app.selected_puzzle().unwrap();
        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(Some("short description".to_owned())),
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();

        assert_eq!(symbol_y(terminal.backend().buffer(), "▐"), None);
    }

    #[test]
    fn all_tabs_render() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();

        app.update(Action::NextTab);
        assert_eq!(app.active_tab, Tab::Language);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Packages"));
        assert!(rendered.contains("Libraries"));
        assert!(rendered.contains("t/T template"));

        app.update(Action::NextTab);
        assert_eq!(app.active_tab, Tab::Config);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        assert!(buffer_text(terminal.backend().buffer()).contains("Configuration management"));
    }

    #[test]
    fn language_tab_renders_lists_and_cancel_default_confirmation() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::NextTab);
        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Ok(LanguageData {
                packages: vec!["anyhow".to_owned()],
                libraries: vec!["grid".to_owned()],
            }),
        });
        app.update(Action::RemoveLanguageItem);

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("anyhow"));
        assert!(rendered.contains("grid"));
        assert!(rendered.contains("Remove package anyhow?"));
        assert!(rendered.contains("[ Cancel ]"));
        assert!(rendered.contains("[ Remove ]"));
    }

    #[test]
    fn language_activity_uses_pane_borders_and_errors_use_a_popup() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::NextTab);

        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Packages - loading..."));
        assert!(rendered.contains("Libraries - loading..."));
        assert!(!rendered.contains("Result"));

        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Err("package query failed".to_owned()),
        });
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Error"));
        assert!(rendered.contains("package query failed"));
        assert!(rendered.contains("Enter/Esc dismiss"));
    }

    #[test]
    fn language_tab_renders_in_a_narrow_terminal() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::NextTab);

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Rust"));
        assert!(rendered.contains("Packages"));
        assert!(rendered.contains("Libraries"));
        assert!(!rendered.contains("Result"));
    }

    #[test]
    fn language_lists_keep_the_selected_item_visible() {
        let backend = TestBackend::new(60, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::NextTab);
        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Ok(LanguageData {
                packages: (0..20).map(|index| format!("package-{index:02}")).collect(),
                libraries: vec![],
            }),
        });
        app.language_package_selection = 19;

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("package-19"));
        assert!(!rendered.contains("package-00"));
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

    #[test]
    fn selected_multiline_puzzle_highlights_only_its_final_row() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let selected = PuzzleId::new(PuzzleDay::new(8).unwrap(), PuzzleYear::new(2026).unwrap());
        let other = PuzzleId::new(PuzzleDay::new(10).unwrap(), PuzzleYear::new(2026).unwrap());
        app.update(Action::CalendarFinished {
            year: other.year,
            refresh: false,
            result: Ok(Calendar {
                rows: vec![
                    calendar_row("D", None),
                    calendar_row("@", Some(selected)),
                    calendar_row("#", Some(selected)),
                    calendar_row("&", Some(other)),
                ],
            }),
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let buffer = terminal.backend().buffer();
        assert_ne!(symbol_background(buffer, "@"), Some(Color::DarkGray));
        assert_eq!(symbol_background(buffer, "#"), Some(Color::DarkGray));
        assert_ne!(symbol_background(buffer, "&"), Some(Color::DarkGray));
    }

    fn calendar_row(text: &str, puzzle: Option<PuzzleId>) -> CalendarRow {
        CalendarRow {
            cells: vec![CalendarCell {
                text: text.to_owned(),
                color: Rgb {
                    red: 255,
                    green: 255,
                    blue: 255,
                },
                stars: None,
                puzzle,
            }],
        }
    }

    fn symbol_background(buffer: &ratatui::buffer::Buffer, symbol: &str) -> Option<Color> {
        let area = buffer.area;
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                let cell = &buffer[(x, y)];
                if cell.symbol() == symbol {
                    return Some(cell.bg);
                }
            }
        }
        None
    }

    fn symbol_y(buffer: &ratatui::buffer::Buffer, symbol: &str) -> Option<u16> {
        let area = buffer.area;
        for y in area.y..area.y + area.height {
            for x in area.x..area.x + area.width {
                if buffer[(x, y)].symbol() == symbol {
                    return Some(y);
                }
            }
        }
        None
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
