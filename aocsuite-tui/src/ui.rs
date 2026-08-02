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
    friendly_puzzle, App, ConfigDialog, ConfigField, ConfigOperationState, DescriptionState,
    LanguageConfirmation, LanguageDialog, LanguageFocus, LanguageOperationState, LanguageTextInput,
    RunDialog, RunInput, RunRequest, Tab,
};

pub fn render(frame: &mut Frame<'_>, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(frame.area());
    render_tabs(frame, root[0], app);
    match app.active_tab {
        Tab::Calendar => render_calendar_tab(frame, root[1], app),
        Tab::Language => render_language_tab(frame, root[1], app),
        Tab::Config => render_config_tab(frame, root[1], app),
    }
    render_footer(frame, root[2], app);
    if let Some(request) = app.active_run {
        render_running(frame, request, app.run_spinner_frame);
    } else if let Some(dialog) = &app.run_dialog {
        render_run_dialog(frame, dialog);
    } else if let Some(dialog) = &app.config_dialog {
        render_config_dialog(frame, dialog);
    } else if let Some(dialog) = &app.language_dialog {
        render_language_dialog(frame, dialog);
    } else if app.help_open {
        render_help_dialog(frame, app);
    }
}

fn run_context(request: RunRequest, include_input: bool) -> String {
    let mut context = format!(
        "{}  |  {}  |  Part {}",
        friendly_puzzle(request.puzzle),
        request.language,
        request.part
    );
    if include_input {
        context.push_str(match request.input {
            RunInput::Aoc => "  |  AoC input",
            RunInput::Example => "  |  Shared example",
        });
    }
    context
}

fn run_title(label: &str, request: RunRequest, width: u16) -> String {
    let title = format!(" {label} | {} ", run_context(request, true));
    let maximum = usize::from(width.saturating_sub(4));
    if title.chars().count() <= maximum {
        return title;
    }
    if maximum <= 3 {
        return ".".repeat(maximum);
    }
    let mut truncated = title.chars().take(maximum - 3).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn render_running(frame: &mut Frame<'_>, request: RunRequest, spinner_frame: usize) {
    let full = frame.area();
    let width = full.width.saturating_sub(4).min(84);
    let height = full.height.saturating_sub(2).min(7);
    let area = Rect::new(
        full.x + full.width.saturating_sub(width) / 2,
        full.y + full.height.saturating_sub(height) / 2,
        width,
        height,
    );
    if area.width < 4 || area.height < 4 {
        return;
    }
    let spinner = ["|", "/", "-", "\\"][spinner_frame % 4];
    let title = run_title("Running", request, area.width);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Line::styled(
            spinner,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}

fn render_run_dialog(frame: &mut Frame<'_>, dialog: &RunDialog) {
    let full = frame.area();
    let width = full.width.saturating_sub(4).min(84);
    let height = full.height.saturating_sub(2).min(22);
    let area = Rect::new(
        full.x + full.width.saturating_sub(width) / 2,
        full.y + full.height.saturating_sub(height) / 2,
        width,
        height,
    );
    if area.width < 4 || area.height < 4 {
        return;
    }
    frame.render_widget(Clear, area);
    match dialog {
        RunDialog::Result {
            request,
            result,
            scroll,
        } => {
            let mut text = String::new();
            let label;
            match result {
                Err(failure) => {
                    label = "Run failed";
                    text.push_str(&format!("{}\n", failure.summary));
                    if let Some(details) = &failure.details {
                        text.push_str(&format!("\n{details}"));
                    }
                }
                Ok(report) => {
                    label = "Run result";
                    if let Some(part) = report.parts.first() {
                        text.push_str(&format!(
                            "Answer\n{}\n\nRuntime\n{} ms\n",
                            part.answer, part.runtime_ms
                        ));
                    }
                    for (label, output) in [
                        ("Compile stdout", &report.compile_stdout),
                        ("Compile stderr", &report.compile_stderr),
                        ("Solver stdout", &report.solver_stdout),
                        ("Solver stderr", &report.solver_stderr),
                    ] {
                        if !output.is_empty() {
                            text.push_str(&format!("\n{label}\n{output}"));
                        }
                    }
                    if let Some(warning) = &report.warning {
                        text.push_str(&format!("\nWarning\n{warning}"));
                    }
                }
            }
            let title = run_title(label, *request, area.width);
            frame.render_widget(
                Paragraph::new(text)
                    .scroll((*scroll, 0))
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title(title)),
                area,
            );
        }
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
        [Constraint::Length(60), Constraint::Fill(1)]
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
        None => vec![Line::from("Calendar unavailable. Press u to retry.")],
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
                    input_line(value),
                    Line::styled(
                        error.as_deref().unwrap_or_default(),
                        Style::default().fg(Color::Red),
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
                ])
                .block(Block::default().borders(Borders::ALL).title(title)),
                area,
            );
        }
        LanguageDialog::Message(message) => {
            frame.render_widget(
                Paragraph::new(vec![Line::from(message.as_str())])
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title(" Error ")),
                area,
            );
        }
    }
}

fn render_config_tab(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let title = match app.config_operation {
        ConfigOperationState::Idle => " Config ".to_owned(),
        ConfigOperationState::Running(activity) => format!(" Config - {activity} "),
    };
    let lines = ConfigField::ALL
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let selected = index == app.config_selection;
            let value = app
                .config
                .as_ref()
                .map_or_else(String::new, |config| match field {
                    ConfigField::Year => config.year.clone(),
                    ConfigField::Editor => config
                        .editor
                        .clone()
                        .unwrap_or_else(|| "Not configured".to_owned()),
                    ConfigField::RunHistoryLimit => config.run_history_limit.clone(),
                    ConfigField::Session if config.session_configured => "Configured".to_owned(),
                    ConfigField::Session => "Not configured".to_owned(),
                });
            let style = if selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::styled(
                format!(
                    "{} {:<24} {value}",
                    if selected { ">" } else { " " },
                    field.label()
                ),
                style,
            )
        })
        .collect::<Vec<_>>();
    let visible_rows = usize::from(area.height.saturating_sub(2));
    let offset = app
        .config_selection
        .saturating_add(1)
        .saturating_sub(visible_rows)
        .min(u16::MAX as usize) as u16;
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((offset, 0))
            .block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
}

fn render_config_dialog(frame: &mut Frame<'_>, dialog: &ConfigDialog) {
    let area = centered_dialog(frame.area());
    if area.width < 4 || area.height < 4 {
        return;
    }
    frame.render_widget(Clear, area);
    match dialog {
        ConfigDialog::Text {
            field,
            value,
            error,
        } => {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(field.label()),
                    input_line(value),
                    Line::styled(
                        error.as_deref().unwrap_or_default(),
                        Style::default().fg(Color::Red),
                    ),
                ])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Edit setting "),
                ),
                area,
            );
        }
        ConfigDialog::Session { value, error } => {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from("Set or replace session"),
                    input_line(if value.is_empty() { "" } else { "********" }),
                    Line::styled(
                        error.as_deref().unwrap_or_default(),
                        Style::default().fg(Color::Red),
                    ),
                ])
                .block(Block::default().borders(Borders::ALL).title(" Session ")),
                area,
            );
        }
        ConfigDialog::ConfirmRemoveSession { confirmed } => {
            let cancel_style = if *confirmed {
                Style::default()
            } else {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            };
            let remove_style = if *confirmed {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from("Remove the configured session?"),
                    Line::default(),
                    Line::from(vec![
                        Span::styled("[ Cancel ]", cancel_style),
                        Span::raw("   "),
                        Span::styled("[ Remove ]", remove_style),
                    ]),
                ])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Remove session "),
                ),
                area,
            );
        }
        ConfigDialog::Message { message, scroll } => {
            let block = Block::default().borders(Borders::ALL).title(" Error ");
            let inner = block.inner(area);
            frame.render_widget(block, area);
            frame.render_widget(
                Paragraph::new(message.as_str())
                    .scroll((*scroll, 0))
                    .wrap(Wrap { trim: false }),
                inner,
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

fn render_footer(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let text = match &app.status {
        Some(status) => format!("? help\n{status}"),
        None => "? help".to_owned(),
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::Gray)),
        area,
    );
}

fn render_help_dialog(frame: &mut Frame<'_>, app: &App) {
    let area = help_dialog_area(frame.area());
    if area.width < 4 || area.height < 4 {
        return;
    }
    let mut lines = vec![
        key_line("q", "Quit", area.width),
        key_line("Tab / Shift-Tab", "Next / previous tab", area.width),
        Line::default(),
    ];
    match app.active_tab {
        Tab::Calendar => lines.extend([
            key_line("Up / Down", "Select puzzle", area.width),
            key_line("Left / Right", "Previous / next year", area.width),
            key_line("Ctrl + arrows", "Pan calendar", area.width),
            key_line("PageUp / PageDown", "Scroll puzzle description", area.width),
            key_line("d", "Download or refresh puzzle description", area.width),
            key_line("1 / 2", "Run puzzle part one / two", area.width),
            key_line("i", "Toggle AoC / shared-example input", area.width),
            key_line("u", "Refresh calendar", area.width),
            key_line("b", "Open puzzle in browser", area.width),
            key_line("Enter", "Open exercise in editor", area.width),
        ]),
        Tab::Language => lines.extend([
            key_line("s", "Switch session language", area.width),
            key_line(
                "Left / Right",
                "Select packages / libraries pane",
                area.width,
            ),
            key_line("Up / Down", "Select package or library", area.width),
            key_line("r", "Reload package and library lists", area.width),
            key_line("a", "Add package", area.width),
            key_line("x", "Remove selected package or library", area.width),
            key_line("n", "Create library", area.width),
            key_line("Enter", "Open selected library", area.width),
            key_line("t / T", "Open / reset template", area.width),
        ]),
        Tab::Config => lines.extend([
            key_line("Up / Down", "Select configuration field", area.width),
            key_line("Enter", "Edit selected field", area.width),
            key_line("r", "Reload configuration", area.width),
            key_line("x", "Reset field / remove session", area.width),
        ]),
    }
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} keymap ", app.active_tab.title())),
            )
            .scroll((app.help_scroll, 0))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn key_line(key: &'static str, description: &'static str, width: u16) -> Line<'static> {
    let key_width = if width < 50 { 12 } else { 20 };
    Line::from(vec![
        Span::styled(
            format!("{key:<key_width$}"),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(description),
    ])
}

fn input_line(value: &str) -> Line<'_> {
    Line::from(format!("> {value}"))
}

fn help_dialog_area(area: Rect) -> Rect {
    let width = area.width.saturating_sub(4).min(72);
    let height = area.height.saturating_sub(2).min(18);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
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
    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear};
    use ratatui::{backend::TestBackend, style::Color, Terminal};

    use crate::app::{
        Action, App, ConfigData, LanguageData, RunDialog, RunFailure, RunInput, RunPartReport,
        RunReport, RunRequest, SecretCharacter, Tab,
    };

    use super::{render, run_title};

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
        assert!(rendered.contains("? help"));
        assert!(!rendered.contains("d download"));
    }

    #[test]
    fn running_and_scrolled_result_render_friendly_context() {
        let backend = TestBackend::new(100, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let request = RunRequest {
            puzzle: app.selected_puzzle().unwrap(),
            language: LanguageId::Rust,
            part: PuzzlePart::One,
            input: RunInput::Aoc,
        };
        app.active_run = Some(request);
        app.run_spinner_frame = 1;
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Running"));
        assert!(rendered.contains("/"));
        assert!(rendered.contains("2026 Day 10"));
        assert!(rendered.contains("rust"));
        assert!(rendered.contains("Part 1"));
        assert!(rendered.contains("AoC input"));

        app.active_run = None;
        app.run_dialog = Some(RunDialog::Result {
            request,
            result: Ok(RunReport {
                request,
                compile_stdout: "compile marker".to_owned(),
                compile_stderr: String::new(),
                solver_stdout: "line\n".repeat(20) + "solver marker",
                solver_stderr: String::new(),
                parts: vec![RunPartReport {
                    part: PuzzlePart::One,
                    answer: "42".to_owned(),
                    runtime_ms: 7,
                }],
                warning: Some("timing warning".to_owned()),
            }),
            scroll: 0,
        });
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Run result"));
        assert!(rendered.contains("2026 Day 10"));
        assert!(rendered.contains("Answer"));
        assert!(rendered.contains("42"));
        assert!(rendered.contains("Runtime"));

        let Some(RunDialog::Result { scroll, .. }) = &mut app.run_dialog else {
            unreachable!();
        };
        *scroll = 22;
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("line"));
        assert!(!rendered.contains("compile marker"));
    }

    #[test]
    fn run_failure_renders_summary_details_and_friendly_context() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        let request = RunRequest {
            puzzle: app.selected_puzzle().unwrap(),
            language: LanguageId::Rust,
            part: PuzzlePart::Two,
            input: RunInput::Example,
        };
        app.run_dialog = Some(RunDialog::Result {
            request,
            result: Err(RunFailure {
                request,
                summary: "Solver command exited with status 1".to_owned(),
                details: Some("compiler rejected the solution".to_owned()),
            }),
            scroll: 0,
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Run failed"));
        assert!(rendered.contains("2026 Day 10"));
        assert!(rendered.contains("Part 2"));
        assert!(rendered.contains("Shared example"));
        assert!(rendered.contains("compiler rejected the solution"));
    }

    #[test]
    fn run_title_truncates_to_the_available_border_width() {
        let request = RunRequest {
            puzzle: PuzzleId::new(PuzzleDay::new(10).unwrap(), PuzzleYear::new(2026).unwrap()),
            language: LanguageId::Rust,
            part: PuzzlePart::Two,
            input: RunInput::Example,
        };

        let title = run_title("Run failed", request, 24);

        assert_eq!(title.chars().count(), 20);
        assert!(title.ends_with("..."));
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
        assert!(rendered.contains("? help"));
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
        assert!(buffer_text(terminal.backend().buffer()).contains("? help"));

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
        assert!(rendered.contains("? help"));

        app.update(Action::NextTab);
        assert_eq!(app.active_tab, Tab::Config);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Default year"));
        assert!(rendered.contains("Editor executable"));
        assert!(rendered.contains("Run-history retention"));
        assert!(rendered.contains("Session"));
    }

    #[test]
    fn help_popup_is_hidden_until_requested_and_is_tab_specific() {
        let backend = TestBackend::new(100, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();

        terminal.draw(|frame| render(frame, &app)).unwrap();
        assert!(!buffer_text(terminal.backend().buffer()).contains("Download or refresh"));

        app.update(Action::OpenHelp);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Calendar keymap"));
        assert!(rendered.contains("Download or refresh"));

        app.update(Action::CloseHelp);
        app.update(Action::NextTab);
        app.update(Action::OpenHelp);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Language keymap"));
        assert!(rendered.contains("Add package"));

        app.update(Action::CloseHelp);
        app.update(Action::NextTab);
        app.update(Action::OpenHelp);
        terminal.draw(|frame| render(frame, &app)).unwrap();
        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Config keymap"));
        assert!(rendered.contains("Edit selected field"));
    }

    #[test]
    fn help_popup_scrolls_in_a_short_narrow_terminal() {
        let backend = TestBackend::new(36, 9);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::OpenHelp);

        for _ in 0..8 {
            app.update(Action::ScrollHelpDown);
        }
        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("Calendar keymap"));
        assert!(rendered.contains("Download"));
    }

    #[test]
    fn visible_modal_matches_input_priority() {
        let backend = TestBackend::new(70, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::OpenHelp);
        app.config_dialog = Some(crate::app::ConfigDialog::Message {
            message: "config failed".to_owned(),
            scroll: 0,
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("config failed"));
        assert!(!rendered.contains("Calendar keymap"));
    }

    #[test]
    fn long_config_errors_scroll_in_a_narrow_terminal() {
        let backend = TestBackend::new(36, 9);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.config_dialog = Some(crate::app::ConfigDialog::Message {
            message: format!("{}final-marker", "context ".repeat(24)),
            scroll: 5,
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();

        assert!(buffer_text(terminal.backend().buffer()).contains("final-marker"));
    }

    #[test]
    fn config_tab_renders_fields_without_a_language_setting() {
        let backend = TestBackend::new(70, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::PreviousTab);
        app.update(Action::ConfigLoaded {
            result: Ok(ConfigData {
                year: "2026".to_owned(),
                editor: None,
                run_history_limit: "10".to_owned(),
                session_configured: false,
            }),
        });

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(rendered.contains("2026"));
        assert!(rendered.contains("Not configured"));
        assert!(!rendered.contains("Default language"));
    }

    #[test]
    fn short_config_pane_keeps_the_selected_field_visible() {
        let backend = TestBackend::new(36, 9);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::PreviousTab);
        app.update(Action::ConfigLoaded {
            result: Ok(ConfigData {
                year: "2026".to_owned(),
                editor: Some("vim".to_owned()),
                run_history_limit: "10".to_owned(),
                session_configured: true,
            }),
        });
        app.config_selection = 3;

        terminal.draw(|frame| render(frame, &app)).unwrap();

        assert!(buffer_text(terminal.backend().buffer()).contains("> Session"));
    }

    #[test]
    fn session_input_is_masked_in_the_rendered_buffer() {
        let backend = TestBackend::new(70, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = app();
        app.update(Action::PreviousTab);
        app.update(Action::ConfigLoaded {
            result: Ok(ConfigData {
                year: "2026".to_owned(),
                editor: Some("vim".to_owned()),
                run_history_limit: "10".to_owned(),
                session_configured: false,
            }),
        });
        app.config_selection = 3;
        app.update(Action::EditConfigField);
        for character in "sensitive-value".chars() {
            app.update(Action::ConfigSecretInput(SecretCharacter(character)));
        }

        terminal.draw(|frame| render(frame, &app)).unwrap();

        let rendered = buffer_text(terminal.backend().buffer());
        assert!(!rendered.contains("sensitive-value"));
        assert!(rendered.contains("> ********"));
        assert!(!rendered.contains("***************"));
        assert!(rendered.contains("Set or replace session"));
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
        assert!(!rendered.contains("dismiss"));
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
