use std::{collections::HashSet, path::PathBuf};

use aocsuite_parser::Calendar;
use aocsuite_utils::{LanguageId, PuzzleId, PuzzleYear};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Calendar,
    Language,
    Config,
}

impl Tab {
    pub const ALL: [Self; 3] = [Self::Calendar, Self::Language, Self::Config];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Calendar => "Calendar",
            Self::Language => "Language",
            Self::Config => "Config",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Calendar => Self::Language,
            Self::Language => Self::Config,
            Self::Config => Self::Calendar,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Calendar => Self::Config,
            Self::Language => Self::Calendar,
            Self::Config => Self::Language,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptionState {
    Empty,
    Loaded { puzzle: PuzzleId, markdown: String },
    Error { puzzle: PuzzleId, message: String },
}

pub struct App {
    pub active_tab: Tab,
    pub calendar: Option<Calendar>,
    pub calendar_loading: bool,
    pub selected_year: PuzzleYear,
    selected_puzzle: Option<PuzzleId>,
    pub latest_puzzle: PuzzleId,
    pub description: DescriptionState,
    pub description_scroll: u16,
    description_downloads: HashSet<PuzzleId>,
    pub calendar_scroll: (u16, u16),
    pub exercise_preparing: bool,
    pub language: LanguageId,
    pub status: Option<String>,
    pub should_quit: bool,
}

#[derive(Debug)]
pub enum Action {
    Quit,
    NextTab,
    PreviousTab,
    PreviousYear,
    NextYear,
    PreviousCalendarPuzzle,
    NextCalendarPuzzle,
    DownloadDescription,
    RefreshCalendar,
    OpenBrowser,
    OpenExercise,
    ScrollDescriptionUp,
    ScrollDescriptionDown,
    ScrollCalendarUp,
    ScrollCalendarDown,
    ScrollCalendarLeft,
    ScrollCalendarRight,
    CalendarFinished {
        year: PuzzleYear,
        refresh: bool,
        result: Result<Calendar, String>,
    },
    CachedDescriptionFinished {
        puzzle: PuzzleId,
        result: Result<Option<String>, String>,
    },
    DescriptionDownloaded {
        puzzle: PuzzleId,
        result: Result<String, String>,
    },
    ExercisePrepared {
        puzzle: PuzzleId,
        result: Result<PreparedExercise, String>,
    },
    ForegroundFinished(Result<(), String>),
    EffectFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundEffect {
    LoadCalendar { year: PuzzleYear, refresh: bool },
    LoadCachedDescription(PuzzleId),
    DownloadDescription(PuzzleId),
    PrepareExercise(PuzzleId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForegroundEffect {
    OpenBrowser(PuzzleId),
    OpenExercise(PreparedExercise),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedExercise {
    pub puzzle: PuzzleId,
    pub editor: String,
    pub puzzle_description: PathBuf,
    pub example: PathBuf,
    pub solution: PathBuf,
    pub input: PathBuf,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Background(BackgroundEffect),
    Foreground(ForegroundEffect),
}

impl App {
    pub fn new(
        configured_year: Option<PuzzleYear>,
        latest_puzzle: PuzzleId,
        language: LanguageId,
    ) -> Self {
        let selected_year = configured_year
            .filter(|year| *year <= latest_puzzle.year)
            .unwrap_or(latest_puzzle.year);
        Self {
            active_tab: Tab::Calendar,
            calendar: None,
            calendar_loading: false,
            selected_year,
            selected_puzzle: None,
            latest_puzzle,
            description: DescriptionState::Empty,
            description_scroll: 0,
            description_downloads: HashSet::new(),
            calendar_scroll: (0, 0),
            exercise_preparing: false,
            language,
            status: None,
            should_quit: false,
        }
    }

    pub fn initial_effects(&mut self) -> Vec<Effect> {
        self.calendar_loading = true;
        vec![Effect::Background(BackgroundEffect::LoadCalendar {
            year: self.selected_year,
            refresh: false,
        })]
    }

    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::NextTab => self.active_tab = self.active_tab.next(),
            Action::PreviousTab => self.active_tab = self.active_tab.previous(),
            Action::PreviousYear if self.active_tab == Tab::Calendar => {
                if self.selected_year.get() > PuzzleYear::MIN {
                    let year = PuzzleYear::new(self.selected_year.get() - 1)
                        .expect("previous released year is valid");
                    return self.select_year(year);
                }
            }
            Action::NextYear if self.active_tab == Tab::Calendar => {
                if self.selected_year < self.latest_puzzle.year {
                    let year = PuzzleYear::new(self.selected_year.get() + 1)
                        .expect("next released year is valid");
                    return self.select_year(year);
                }
            }
            Action::PreviousCalendarPuzzle if self.active_tab == Tab::Calendar => {
                return self.move_calendar_selection(-1);
            }
            Action::NextCalendarPuzzle if self.active_tab == Tab::Calendar => {
                return self.move_calendar_selection(1);
            }
            Action::DownloadDescription if self.active_tab == Tab::Calendar => {
                let Some(puzzle) = self.selected_puzzle_or_status() else {
                    return Vec::new();
                };
                if !self.description_downloads.insert(puzzle) {
                    return Vec::new();
                }
                self.status = Some(format!("Downloading {puzzle}"));
                return vec![Effect::Background(BackgroundEffect::DownloadDescription(
                    puzzle,
                ))];
            }
            Action::RefreshCalendar if self.active_tab == Tab::Calendar => {
                self.calendar_loading = true;
                self.status = Some(format!("Refreshing calendar {}", self.selected_year));
                return vec![Effect::Background(BackgroundEffect::LoadCalendar {
                    year: self.selected_year,
                    refresh: true,
                })];
            }
            Action::OpenBrowser if self.active_tab == Tab::Calendar => {
                let Some(puzzle) = self.selected_puzzle_or_status() else {
                    return Vec::new();
                };
                return vec![Effect::Foreground(ForegroundEffect::OpenBrowser(puzzle))];
            }
            Action::OpenExercise if self.active_tab == Tab::Calendar => {
                if self.exercise_preparing {
                    self.status = Some("An exercise is already being prepared".to_owned());
                    return Vec::new();
                }
                let Some(puzzle) = self.selected_puzzle_or_status() else {
                    return Vec::new();
                };
                self.exercise_preparing = true;
                self.status = Some(format!("Preparing {puzzle} for the editor"));
                return vec![Effect::Background(BackgroundEffect::PrepareExercise(
                    puzzle,
                ))];
            }
            Action::ScrollDescriptionUp if self.active_tab == Tab::Calendar => {
                self.description_scroll = self.description_scroll.saturating_sub(1);
            }
            Action::ScrollDescriptionDown if self.active_tab == Tab::Calendar => {
                if matches!(self.description, DescriptionState::Loaded { .. }) {
                    self.description_scroll = self.description_scroll.saturating_add(1);
                }
            }
            Action::ScrollCalendarUp if self.active_tab == Tab::Calendar => {
                self.calendar_scroll.0 = self.calendar_scroll.0.saturating_sub(1);
            }
            Action::ScrollCalendarDown if self.active_tab == Tab::Calendar => {
                self.calendar_scroll.0 = self.calendar_scroll.0.saturating_add(1);
            }
            Action::ScrollCalendarLeft if self.active_tab == Tab::Calendar => {
                self.calendar_scroll.1 = self.calendar_scroll.1.saturating_sub(2);
            }
            Action::ScrollCalendarRight if self.active_tab == Tab::Calendar => {
                self.calendar_scroll.1 = self.calendar_scroll.1.saturating_add(2);
            }
            Action::CalendarFinished {
                year,
                refresh,
                result,
            } => {
                if year != self.selected_year {
                    return Vec::new();
                }
                self.calendar_loading = false;
                match result {
                    Ok(calendar) => {
                        let previous_puzzle = self.selected_puzzle;
                        let available = calendar_puzzles(&calendar);
                        let preferred = if refresh { previous_puzzle } else { None };
                        let selected = preferred
                            .filter(|puzzle| available.contains(puzzle))
                            .or_else(|| available.first().copied());
                        self.calendar = Some(calendar);
                        self.selected_puzzle = selected;
                        let mut effects = Vec::new();
                        if selected != previous_puzzle {
                            self.clear_description();
                            if let Some(puzzle) = selected {
                                effects.push(self.load_cached_description(puzzle));
                            }
                        }
                        self.status = Some(if refresh {
                            format!("Refreshed calendar {year}")
                        } else {
                            format!("Loaded calendar {year}")
                        });
                        return effects;
                    }
                    Err(message) => self.status = Some(message),
                }
            }
            Action::CachedDescriptionFinished { puzzle, result } => {
                if Some(puzzle) != self.selected_puzzle() {
                    return Vec::new();
                }
                match result {
                    Ok(Some(markdown)) => {
                        self.description = DescriptionState::Loaded { puzzle, markdown };
                    }
                    Ok(None) => {}
                    Err(message) => {
                        self.status = Some(message.clone());
                        if !matches!(self.description, DescriptionState::Loaded { .. }) {
                            self.description = DescriptionState::Error { puzzle, message };
                        }
                    }
                }
            }
            Action::DescriptionDownloaded { puzzle, result } => {
                self.description_downloads.remove(&puzzle);
                if Some(puzzle) != self.selected_puzzle() {
                    return Vec::new();
                }
                match result {
                    Ok(markdown) => {
                        self.description = DescriptionState::Loaded { puzzle, markdown };
                        self.description_scroll = 0;
                        self.status = Some(format!("Downloaded {puzzle}"));
                    }
                    Err(message) => {
                        self.status = Some(message.clone());
                        if !matches!(self.description, DescriptionState::Loaded { .. }) {
                            self.description = DescriptionState::Error { puzzle, message };
                        }
                    }
                }
            }
            Action::ExercisePrepared { puzzle, result } => {
                self.exercise_preparing = false;
                if Some(puzzle) != self.selected_puzzle() {
                    return Vec::new();
                }
                match result {
                    Ok(prepared) => {
                        self.status = Some(format!("Opening {puzzle} in the editor"));
                        return vec![Effect::Foreground(ForegroundEffect::OpenExercise(prepared))];
                    }
                    Err(message) => self.status = Some(message),
                }
            }
            Action::ForegroundFinished(result) => {
                self.status = Some(match result {
                    Ok(()) => "Returned from external application".to_owned(),
                    Err(message) => message,
                });
            }
            Action::EffectFailed(message) => self.status = Some(message),
            _ => {}
        }
        Vec::new()
    }

    pub const fn selected_puzzle(&self) -> Option<PuzzleId> {
        self.selected_puzzle
    }

    pub fn description_downloading(&self, puzzle: PuzzleId) -> bool {
        self.description_downloads.contains(&puzzle)
    }

    fn select_year(&mut self, year: PuzzleYear) -> Vec<Effect> {
        self.selected_year = year;
        self.selected_puzzle = None;
        self.calendar = None;
        self.calendar_loading = true;
        self.calendar_scroll = (0, 0);
        self.clear_description();
        self.status = Some(format!("Loading calendar {year}"));
        vec![Effect::Background(BackgroundEffect::LoadCalendar {
            year,
            refresh: false,
        })]
    }

    fn move_calendar_selection(&mut self, direction: i8) -> Vec<Effect> {
        let available = self
            .calendar
            .as_ref()
            .map(calendar_puzzles)
            .unwrap_or_default();
        let current = self
            .selected_puzzle
            .and_then(|puzzle| available.iter().position(|candidate| *candidate == puzzle));
        let next = match (current, direction) {
            (Some(index), direction) if direction < 0 => index.checked_sub(1),
            (Some(index), direction) if direction > 0 && index + 1 < available.len() => {
                Some(index + 1)
            }
            (None, direction) if direction < 0 => available.len().checked_sub(1),
            (None, direction) if direction > 0 && !available.is_empty() => Some(0),
            _ => None,
        };
        let Some(puzzle) = next.map(|index| available[index]) else {
            return Vec::new();
        };

        self.selected_puzzle = Some(puzzle);
        self.clear_description();
        vec![self.load_cached_description(puzzle)]
    }

    fn selected_puzzle_or_status(&mut self) -> Option<PuzzleId> {
        if self.selected_puzzle.is_none() {
            self.status = Some("No calendar puzzle is selected".to_owned());
        }
        self.selected_puzzle
    }

    fn load_cached_description(&self, puzzle: PuzzleId) -> Effect {
        Effect::Background(BackgroundEffect::LoadCachedDescription(puzzle))
    }

    fn clear_description(&mut self) {
        self.description = DescriptionState::Empty;
        self.description_scroll = 0;
        self.status = None;
    }
}

fn calendar_puzzles(calendar: &Calendar) -> Vec<PuzzleId> {
    let mut puzzles = Vec::new();
    for puzzle in calendar
        .rows
        .iter()
        .flat_map(|row| &row.cells)
        .filter_map(|cell| cell.puzzle)
    {
        if !puzzles.contains(&puzzle) {
            puzzles.push(puzzle);
        }
    }
    puzzles
}

#[cfg(test)]
mod tests {
    use aocsuite_parser::{Calendar, CalendarCell, CalendarRow, Rgb};
    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzleYear};

    use super::{
        Action, App, BackgroundEffect, DescriptionState, Effect, ForegroundEffect, PreparedExercise,
    };

    fn puzzle(day: u32, year: i32) -> PuzzleId {
        PuzzleId::new(
            PuzzleDay::new(day).expect("valid test day"),
            PuzzleYear::new(year).expect("valid test year"),
        )
    }

    fn app() -> App {
        App::new(None, puzzle(10, 2026), LanguageId::Rust)
    }

    fn calendar(rows: Vec<Vec<Option<PuzzleId>>>) -> Calendar {
        Calendar {
            rows: rows
                .into_iter()
                .map(|puzzles| CalendarRow {
                    cells: puzzles
                        .into_iter()
                        .map(|puzzle| CalendarCell {
                            text: puzzle.map_or_else(|| "decoration".to_owned(), |p| p.to_string()),
                            color: Rgb {
                                red: 255,
                                green: 255,
                                blue: 255,
                            },
                            stars: None,
                            puzzle,
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    fn load_calendar(app: &mut App, calendar: Calendar, refresh: bool) -> Vec<Effect> {
        app.update(Action::CalendarFinished {
            year: app.selected_year,
            refresh,
            result: Ok(calendar),
        })
    }

    fn selected_app() -> App {
        let mut app = app();
        load_calendar(
            &mut app,
            calendar(vec![
                vec![Some(puzzle(10, 2026))],
                vec![Some(puzzle(9, 2026))],
            ]),
            false,
        );
        app
    }

    #[test]
    fn year_navigation_stays_within_released_bounds() {
        let mut app = app();

        assert!(app.update(Action::NextYear).is_empty());
        assert_eq!(app.selected_year, puzzle(1, 2026).year);

        for _ in 0..20 {
            app.update(Action::PreviousYear);
        }
        assert_eq!(app.selected_year.get(), PuzzleYear::MIN);
        assert!(app.update(Action::PreviousYear).is_empty());
    }

    #[test]
    fn calendar_navigation_follows_visual_puzzle_order() {
        let mut app = app();
        let initial_effects = load_calendar(
            &mut app,
            calendar(vec![
                vec![None],
                vec![Some(puzzle(7, 2026))],
                vec![Some(puzzle(7, 2026))],
                vec![None],
                vec![Some(puzzle(3, 2026)), Some(puzzle(10, 2026))],
            ]),
            false,
        );
        assert_eq!(app.selected_puzzle(), Some(puzzle(7, 2026)));
        assert_eq!(
            initial_effects,
            vec![Effect::Background(BackgroundEffect::LoadCachedDescription(
                puzzle(7, 2026)
            ))]
        );
        app.update(Action::CachedDescriptionFinished {
            puzzle: puzzle(7, 2026),
            result: Ok(Some("description".to_owned())),
        });
        app.calendar_scroll.0 = 3;

        let effects = app.update(Action::NextCalendarPuzzle);

        assert_eq!(app.selected_puzzle(), Some(puzzle(3, 2026)));
        assert_eq!(app.description, DescriptionState::Empty);
        assert_eq!(
            effects,
            vec![Effect::Background(BackgroundEffect::LoadCachedDescription(
                puzzle(3, 2026)
            ))]
        );
        assert_eq!(app.calendar_scroll.0, 3);

        app.update(Action::NextCalendarPuzzle);
        assert_eq!(app.selected_puzzle(), Some(puzzle(10, 2026)));
        assert_eq!(app.calendar_scroll.0, 3);
        assert!(app.update(Action::NextCalendarPuzzle).is_empty());

        app.update(Action::PreviousCalendarPuzzle);
        assert_eq!(app.selected_puzzle(), Some(puzzle(3, 2026)));
    }

    #[test]
    fn initial_effect_only_loads_the_calendar() {
        let mut app = app();

        assert_eq!(
            app.initial_effects(),
            vec![Effect::Background(BackgroundEffect::LoadCalendar {
                year: PuzzleYear::new(2026).unwrap(),
                refresh: false,
            })]
        );
        assert_eq!(app.selected_puzzle(), None);
    }

    #[test]
    fn calendar_load_selects_the_first_visual_puzzle() {
        let mut app = app();

        let effects = load_calendar(
            &mut app,
            calendar(vec![
                vec![Some(puzzle(8, 2026))],
                vec![None],
                vec![Some(puzzle(4, 2026))],
            ]),
            false,
        );

        assert_eq!(app.selected_puzzle(), Some(puzzle(8, 2026)));
        assert_eq!(app.calendar_scroll.0, 0);
        assert_eq!(
            effects,
            vec![Effect::Background(BackgroundEffect::LoadCachedDescription(
                puzzle(8, 2026)
            ))]
        );
    }

    #[test]
    fn description_download_is_explicit_and_single_flight_per_puzzle() {
        let mut app = selected_app();

        let effects = app.update(Action::DownloadDescription);

        assert_eq!(
            effects,
            vec![Effect::Background(BackgroundEffect::DownloadDescription(
                puzzle(10, 2026)
            ))]
        );
        assert!(app.description_downloading(puzzle(10, 2026)));
        assert!(app.update(Action::DownloadDescription).is_empty());
    }

    #[test]
    fn failed_redownload_preserves_an_existing_preview() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();
        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(Some("existing preview".to_owned())),
        });
        app.update(Action::DownloadDescription);

        app.update(Action::DescriptionDownloaded {
            puzzle,
            result: Err("download failed".to_owned()),
        });

        assert_eq!(
            app.description,
            DescriptionState::Loaded {
                puzzle,
                markdown: "existing preview".to_owned()
            }
        );
        assert_eq!(app.status.as_deref(), Some("download failed"));
        assert!(!app.description_downloading(puzzle));
    }

    #[test]
    fn failed_download_without_a_preview_shows_the_error() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();
        app.update(Action::DownloadDescription);

        app.update(Action::DescriptionDownloaded {
            puzzle,
            result: Err("download failed".to_owned()),
        });

        assert_eq!(
            app.description,
            DescriptionState::Error {
                puzzle,
                message: "download failed".to_owned()
            }
        );
    }

    #[test]
    fn stale_download_updates_no_visible_state_and_releases_its_guard() {
        let mut app = selected_app();
        let stale = app.selected_puzzle().unwrap();
        app.update(Action::DownloadDescription);
        app.update(Action::NextCalendarPuzzle);

        app.update(Action::DescriptionDownloaded {
            puzzle: stale,
            result: Ok("updated stale puzzle".to_owned()),
        });

        assert_eq!(app.description, DescriptionState::Empty);
        assert!(!app.description_downloading(stale));
        assert_eq!(app.selected_puzzle(), Some(puzzle(9, 2026)));
        assert!(app.status.is_none());
    }

    #[test]
    fn cache_misses_are_silent_but_cache_errors_are_visible() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();
        let status = app.status.clone();

        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(None),
        });
        assert_eq!(app.description, DescriptionState::Empty);
        assert_eq!(app.status, status);

        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Err("cache read failed".to_owned()),
        });
        assert_eq!(
            app.description,
            DescriptionState::Error {
                puzzle,
                message: "cache read failed".to_owned()
            }
        );
        assert_eq!(app.status.as_deref(), Some("cache read failed"));
    }

    #[test]
    fn stale_calendar_results_do_not_replace_the_selected_year() {
        let mut app = app();

        app.update(Action::PreviousYear);
        app.update(Action::CalendarFinished {
            year: puzzle(1, 2026).year,
            refresh: false,
            result: Ok(Calendar { rows: Vec::new() }),
        });

        assert!(app.calendar.is_none());
        assert_eq!(app.selected_year, puzzle(1, 2025).year);
    }

    #[test]
    fn calendar_selection_starts_at_the_top_visual_puzzle() {
        let mut app = App::new(
            Some(puzzle(1, 2025).year),
            puzzle(25, 2026),
            LanguageId::Python,
        );
        load_calendar(
            &mut app,
            calendar(vec![
                vec![Some(puzzle(11, 2025))],
                vec![Some(puzzle(12, 2025))],
                vec![Some(puzzle(25, 2025))],
            ]),
            false,
        );

        assert_eq!(app.selected_puzzle(), Some(puzzle(11, 2025)));
    }

    #[test]
    fn calendar_refresh_preserves_the_selected_puzzle_in_a_new_position() {
        let mut app = selected_app();
        app.update(Action::NextCalendarPuzzle);
        assert_eq!(app.selected_puzzle(), Some(puzzle(9, 2026)));
        app.calendar_scroll.0 = 4;

        let effects = load_calendar(
            &mut app,
            calendar(vec![
                vec![Some(puzzle(10, 2026))],
                vec![None],
                vec![Some(puzzle(9, 2026))],
            ]),
            true,
        );

        assert!(effects.is_empty());
        assert_eq!(app.selected_puzzle(), Some(puzzle(9, 2026)));
        assert_eq!(app.calendar_scroll.0, 4);
    }

    #[test]
    fn calendar_refresh_falls_back_to_the_first_visual_puzzle() {
        let mut app = selected_app();
        app.calendar_scroll.0 = 2;

        let effects = load_calendar(
            &mut app,
            calendar(vec![
                vec![None],
                vec![Some(puzzle(6, 2026))],
                vec![Some(puzzle(4, 2026))],
            ]),
            true,
        );

        assert_eq!(app.selected_puzzle(), Some(puzzle(6, 2026)));
        assert_eq!(app.calendar_scroll.0, 2);
        assert_eq!(
            effects,
            vec![Effect::Background(BackgroundEffect::LoadCachedDescription(
                puzzle(6, 2026)
            ))]
        );
    }

    #[test]
    fn puzzle_actions_are_disabled_until_the_calendar_selects_a_puzzle() {
        let mut app = app();

        assert!(app.update(Action::DownloadDescription).is_empty());
        assert_eq!(
            app.status.as_deref(),
            Some("No calendar puzzle is selected")
        );
        assert!(app.update(Action::OpenBrowser).is_empty());
        assert!(app.update(Action::OpenExercise).is_empty());
    }

    #[test]
    fn exercise_is_prepared_before_foreground_editor_handoff() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();

        assert_eq!(
            app.update(Action::OpenExercise),
            vec![Effect::Background(BackgroundEffect::PrepareExercise(
                puzzle
            ))]
        );
        assert!(app.exercise_preparing);
        assert!(app.update(Action::OpenExercise).is_empty());

        let prepared = PreparedExercise {
            puzzle,
            editor: "editor".to_owned(),
            puzzle_description: "puzzle.md".into(),
            example: "example.txt".into(),
            solution: "solution.rs".into(),
            input: "input.txt".into(),
            working_directory: "workspace".into(),
        };
        assert_eq!(
            app.update(Action::ExercisePrepared {
                puzzle,
                result: Ok(prepared.clone()),
            }),
            vec![Effect::Foreground(ForegroundEffect::OpenExercise(prepared))]
        );
        assert!(!app.exercise_preparing);
    }

    #[test]
    fn stale_exercise_preparation_releases_the_single_flight_guard() {
        let mut app = selected_app();
        let stale = app.selected_puzzle().unwrap();
        app.update(Action::OpenExercise);
        app.update(Action::NextCalendarPuzzle);

        assert!(app
            .update(Action::ExercisePrepared {
                puzzle: stale,
                result: Err("stale preparation".to_owned()),
            })
            .is_empty());
        assert!(!app.exercise_preparing);
        assert_eq!(app.update(Action::OpenExercise).len(), 1);
    }
}
