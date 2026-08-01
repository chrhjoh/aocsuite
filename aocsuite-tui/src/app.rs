use std::path::PathBuf;

use aocsuite_parser::Calendar;
use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzleYear};

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
    Loading(PuzzleId),
    Loaded { puzzle: PuzzleId, markdown: String },
    Error { puzzle: PuzzleId, message: String },
}

pub struct App {
    pub active_tab: Tab,
    pub calendar: Option<Calendar>,
    pub calendar_loading: bool,
    pub selected_year: PuzzleYear,
    pub selected_day: PuzzleDay,
    pub latest_puzzle: PuzzleId,
    pub description: DescriptionState,
    pub description_scroll: u16,
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
    PreviousDay,
    NextDay,
    LoadDescription,
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
    DescriptionFinished {
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
    LoadDescription(PuzzleId),
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
        let selected_day = final_released_day(selected_year, latest_puzzle);
        Self {
            active_tab: Tab::Calendar,
            calendar: None,
            calendar_loading: false,
            selected_year,
            selected_day,
            latest_puzzle,
            description: DescriptionState::Empty,
            description_scroll: 0,
            calendar_scroll: (0, 0),
            exercise_preparing: false,
            language,
            status: None,
            should_quit: false,
        }
    }

    pub fn initial_effect(&mut self) -> Effect {
        self.calendar_loading = true;
        Effect::Background(BackgroundEffect::LoadCalendar {
            year: self.selected_year,
            refresh: false,
        })
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
                    return vec![self.select_year(year)];
                }
            }
            Action::NextYear if self.active_tab == Tab::Calendar => {
                if self.selected_year < self.latest_puzzle.year {
                    let year = PuzzleYear::new(self.selected_year.get() + 1)
                        .expect("next released year is valid");
                    return vec![self.select_year(year)];
                }
            }
            Action::PreviousDay if self.active_tab == Tab::Calendar => {
                if self.selected_day.get() > PuzzleDay::MIN {
                    self.selected_day = PuzzleDay::new(u32::from(self.selected_day.get() - 1))
                        .expect("previous day is valid");
                    self.clear_description();
                }
            }
            Action::NextDay if self.active_tab == Tab::Calendar => {
                let final_day = final_released_day(self.selected_year, self.latest_puzzle);
                if self.selected_day < final_day {
                    self.selected_day = PuzzleDay::new(u32::from(self.selected_day.get() + 1))
                        .expect("next day is valid");
                    self.clear_description();
                }
            }
            Action::LoadDescription if self.active_tab == Tab::Calendar => {
                let puzzle = self.selected_puzzle();
                self.description = DescriptionState::Loading(puzzle);
                self.description_scroll = 0;
                self.status = Some(format!("Loading {puzzle}"));
                return vec![Effect::Background(BackgroundEffect::LoadDescription(
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
                return vec![Effect::Foreground(ForegroundEffect::OpenBrowser(
                    self.selected_puzzle(),
                ))];
            }
            Action::OpenExercise if self.active_tab == Tab::Calendar => {
                if self.exercise_preparing {
                    self.status = Some("An exercise is already being prepared".to_owned());
                    return Vec::new();
                }
                let puzzle = self.selected_puzzle();
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
                        self.calendar = Some(calendar);
                        self.status = Some(if refresh {
                            format!("Refreshed calendar {year}")
                        } else {
                            format!("Loaded calendar {year}")
                        });
                    }
                    Err(message) => self.status = Some(message),
                }
            }
            Action::DescriptionFinished { puzzle, result } => {
                if puzzle != self.selected_puzzle() {
                    return Vec::new();
                }
                self.description = match result {
                    Ok(markdown) => {
                        self.status = Some(format!("Loaded {puzzle}"));
                        DescriptionState::Loaded { puzzle, markdown }
                    }
                    Err(message) => {
                        self.status = Some(message.clone());
                        DescriptionState::Error { puzzle, message }
                    }
                };
            }
            Action::ExercisePrepared { puzzle, result } => {
                self.exercise_preparing = false;
                if puzzle != self.selected_puzzle() {
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

    pub const fn selected_puzzle(&self) -> PuzzleId {
        PuzzleId::new(self.selected_day, self.selected_year)
    }

    fn select_year(&mut self, year: PuzzleYear) -> Effect {
        self.selected_year = year;
        self.selected_day = final_released_day(year, self.latest_puzzle);
        self.calendar = None;
        self.calendar_loading = true;
        self.calendar_scroll = (0, 0);
        self.clear_description();
        self.status = Some(format!("Loading calendar {year}"));
        Effect::Background(BackgroundEffect::LoadCalendar {
            year,
            refresh: false,
        })
    }

    fn clear_description(&mut self) {
        self.description = DescriptionState::Empty;
        self.description_scroll = 0;
    }
}

fn final_released_day(year: PuzzleYear, latest: PuzzleId) -> PuzzleDay {
    let day = if year == latest.year {
        latest.day.get()
    } else if year.get() == 2025 {
        12
    } else {
        PuzzleDay::MAX
    };
    PuzzleDay::new(u32::from(day)).expect("released final day is valid")
}

#[cfg(test)]
mod tests {
    use aocsuite_parser::Calendar;
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
    fn changing_day_clears_the_loaded_description() {
        let mut app = app();
        let selected = app.selected_puzzle();
        app.update(Action::DescriptionFinished {
            puzzle: selected,
            result: Ok("description".to_owned()),
        });

        app.update(Action::PreviousDay);

        assert_eq!(app.description, DescriptionState::Empty);
    }

    #[test]
    fn description_load_is_explicit() {
        let mut app = app();

        let effects = app.update(Action::LoadDescription);

        assert_eq!(
            effects,
            vec![Effect::Background(BackgroundEffect::LoadDescription(
                puzzle(10, 2026)
            ))]
        );
        assert_eq!(app.description, DescriptionState::Loading(puzzle(10, 2026)));
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
    fn shortened_2025_event_selects_day_twelve() {
        let app = App::new(
            Some(puzzle(1, 2025).year),
            puzzle(25, 2026),
            LanguageId::Python,
        );

        assert_eq!(app.selected_day, puzzle(12, 2025).day);
    }

    #[test]
    fn exercise_is_prepared_before_foreground_editor_handoff() {
        let mut app = app();
        let puzzle = app.selected_puzzle();

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
        let mut app = app();
        let stale = app.selected_puzzle();
        app.update(Action::OpenExercise);
        app.update(Action::PreviousDay);

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
