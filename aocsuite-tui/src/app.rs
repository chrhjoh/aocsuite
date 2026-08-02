use std::{collections::HashSet, path::PathBuf};

use aocsuite_parser::Calendar;
use aocsuite_utils::{LanguageId, PuzzleId, PuzzlePart, PuzzleYear, RunHistoryLimit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunInput {
    Aoc,
    Example,
}

pub(crate) fn friendly_puzzle(puzzle: PuzzleId) -> String {
    format!("{} Day {}", puzzle.year, puzzle.day)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunRequest {
    pub puzzle: PuzzleId,
    pub language: LanguageId,
    pub part: PuzzlePart,
    pub input: RunInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunPartReport {
    pub part: PuzzlePart,
    pub answer: String,
    pub runtime_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub request: RunRequest,
    pub compile_stdout: String,
    pub compile_stderr: String,
    pub solver_stdout: String,
    pub solver_stderr: String,
    pub parts: Vec<RunPartReport>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunFailure {
    pub request: RunRequest,
    pub summary: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunDialog {
    Result {
        request: RunRequest,
        result: Result<RunReport, RunFailure>,
        scroll: u16,
    },
}

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
    CheckingCache(PuzzleId),
    Empty,
    Loaded { puzzle: PuzzleId, markdown: String },
    Error { puzzle: PuzzleId, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageFocus {
    Packages,
    Libraries,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageOperationState {
    Idle,
    Running {
        packages: Option<String>,
        libraries: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageTextInput {
    AddPackage,
    Library,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageConfirmation {
    RemovePackage(String),
    RemoveLibrary(String),
    ResetTemplate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageDialog {
    Text {
        kind: LanguageTextInput,
        value: String,
        error: Option<String>,
    },
    Confirm {
        action: LanguageConfirmation,
        confirmed: bool,
    },
    Message(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageData {
    pub packages: Vec<String>,
    pub libraries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageMutation {
    AddPackage(String),
    RemovePackage(String),
    RemoveLibrary(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageFileKind {
    Library(String),
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedLanguageFile {
    pub language: LanguageId,
    pub kind: LanguageFileKind,
    pub editor: String,
    pub path: PathBuf,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    Year,
    Editor,
    RunHistoryLimit,
    Session,
}

impl ConfigField {
    pub const ALL: [Self; 4] = [
        Self::Year,
        Self::Editor,
        Self::RunHistoryLimit,
        Self::Session,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Year => "Default year",
            Self::Editor => "Editor executable",
            Self::RunHistoryLimit => "Run-history retention",
            Self::Session => "Session",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonSecretConfigField {
    Year,
    Editor,
    RunHistoryLimit,
}

impl NonSecretConfigField {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Year => ConfigField::Year.label(),
            Self::Editor => ConfigField::Editor.label(),
            Self::RunHistoryLimit => ConfigField::RunHistoryLimit.label(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigData {
    pub year: String,
    pub editor: Option<String>,
    pub run_history_limit: String,
    pub session_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigOperationState {
    Idle,
    Running(&'static str),
}

#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(String);

impl SecretString {
    pub fn empty() -> Self {
        Self(String::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    fn push(&mut self, character: char) {
        self.0.push(character);
    }

    fn pop(&mut self) {
        self.0.pop();
    }

    fn trimmed(mut self) -> Option<Self> {
        let trimmed = self.0.trim();
        if trimmed.is_empty() {
            None
        } else {
            self.0 = trimmed.to_owned();
            Some(self)
        }
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretString([REDACTED])")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SecretCharacter(pub(crate) char);

impl std::fmt::Debug for SecretCharacter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretCharacter([REDACTED])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDialog {
    Text {
        field: NonSecretConfigField,
        value: String,
        error: Option<String>,
    },
    Session {
        value: SecretString,
        error: Option<String>,
    },
    ConfirmRemoveSession {
        confirmed: bool,
    },
    Message {
        message: String,
        scroll: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigMutation {
    Set {
        field: NonSecretConfigField,
        value: Option<String>,
    },
    SetSession(SecretString),
    RemoveSession,
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
    pub active_run: Option<RunRequest>,
    pub run_input: RunInput,
    pub run_spinner_frame: usize,
    pub run_dialog: Option<RunDialog>,
    pub language: LanguageId,
    pub language_packages: Vec<String>,
    pub language_libraries: Vec<String>,
    pub language_package_selection: usize,
    pub language_library_selection: usize,
    pub language_focus: LanguageFocus,
    pub language_operation: LanguageOperationState,
    pub language_dialog: Option<LanguageDialog>,
    language_loaded: bool,
    language_file_opening: Option<LanguageFileKind>,
    pub config: Option<ConfigData>,
    pub config_selection: usize,
    pub config_operation: ConfigOperationState,
    pub config_dialog: Option<ConfigDialog>,
    pub help_open: bool,
    pub help_scroll: u16,
    quit_after_config_save: bool,
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
    RunPart(PuzzlePart),
    ToggleRunInput,
    Tick,
    CancelRunDialog,
    ScrollRunUp,
    ScrollRunDown,
    SwitchLanguage,
    RefreshLanguage,
    PreviousLanguagePane,
    NextLanguagePane,
    PreviousLanguageItem,
    NextLanguageItem,
    AddPackage,
    RemoveLanguageItem,
    NewLibrary,
    OpenLanguageItem,
    OpenTemplate,
    ResetTemplate,
    RefreshConfig,
    PreviousConfigField,
    NextConfigField,
    EditConfigField,
    RemoveConfigValue,
    ConfigInput(char),
    ConfigSecretInput(SecretCharacter),
    ConfigBackspace,
    ConfigToggleConfirmation,
    ConfigScrollMessageUp,
    ConfigScrollMessageDown,
    ConfigSubmit,
    ConfigCancel,
    OpenHelp,
    CloseHelp,
    ScrollHelpUp,
    ScrollHelpDown,
    DialogInput(char),
    DialogBackspace,
    DialogToggleConfirmation,
    DialogSubmit,
    DialogCancel,
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
        language: LanguageId,
        result: Result<PreparedExercise, String>,
    },
    RunFinished {
        request: RunRequest,
        result: Result<RunReport, RunFailure>,
    },
    RunEffectFailed {
        request: RunRequest,
        failure: RunFailure,
    },
    LanguageDataFinished {
        language: LanguageId,
        result: Result<LanguageData, String>,
    },
    LanguageMutationFinished {
        language: LanguageId,
        result: Result<LanguageData, String>,
    },
    LanguageFilePrepared {
        language: LanguageId,
        result: Result<PreparedLanguageFile, String>,
    },
    LanguageEffectFailed(String),
    ConfigLoaded {
        result: Result<ConfigData, String>,
    },
    ConfigSaved {
        result: Result<ConfigData, String>,
    },
    ConfigEffectFailed(String),
    ForegroundFinished(Result<(), String>),
    EffectFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundEffect {
    LoadCalendar {
        year: PuzzleYear,
        refresh: bool,
    },
    LoadCachedDescription(PuzzleId),
    DownloadDescription(PuzzleId),
    PrepareExercise {
        puzzle: PuzzleId,
        language: LanguageId,
    },
    RunSolver(RunRequest),
    LoadLanguageData {
        language: LanguageId,
    },
    MutateLanguage {
        language: LanguageId,
        mutation: LanguageMutation,
    },
    PrepareLanguageFile {
        language: LanguageId,
        kind: LanguageFileKind,
        reset: bool,
    },
    LoadConfig {
        latest_year: PuzzleYear,
    },
    MutateConfig {
        latest_year: PuzzleYear,
        mutation: ConfigMutation,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForegroundEffect {
    OpenBrowser(PuzzleId),
    OpenExercise(PreparedExercise),
    OpenLanguageFile(PreparedLanguageFile),
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
            active_run: None,
            run_input: RunInput::Aoc,
            run_spinner_frame: 0,
            run_dialog: None,
            language,
            language_packages: Vec::new(),
            language_libraries: Vec::new(),
            language_package_selection: 0,
            language_library_selection: 0,
            language_focus: LanguageFocus::Packages,
            language_operation: LanguageOperationState::Idle,
            language_dialog: None,
            language_loaded: false,
            language_file_opening: None,
            config: None,
            config_selection: 0,
            config_operation: ConfigOperationState::Idle,
            config_dialog: None,
            help_open: false,
            help_scroll: 0,
            quit_after_config_save: false,
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
        if self.active_run.is_some()
            && !matches!(
                action,
                Action::Tick
                    | Action::RunFinished { .. }
                    | Action::RunEffectFailed { .. }
                    | Action::CalendarFinished { .. }
                    | Action::CachedDescriptionFinished { .. }
                    | Action::DescriptionDownloaded { .. }
                    | Action::ExercisePrepared { .. }
                    | Action::LanguageDataFinished { .. }
                    | Action::LanguageMutationFinished { .. }
                    | Action::LanguageFilePrepared { .. }
                    | Action::LanguageEffectFailed(_)
                    | Action::ConfigLoaded { .. }
                    | Action::ConfigSaved { .. }
                    | Action::ConfigEffectFailed(_)
                    | Action::ForegroundFinished(_)
                    | Action::EffectFailed(_)
            )
        {
            if matches!(action, Action::Quit) {
                self.status = Some("Wait for the solver run to finish".to_owned());
            }
            return Vec::new();
        }
        match action {
            Action::Quit if self.config_saving() => {
                self.quit_after_config_save = true;
                self.status = Some("Wait for the configuration save to finish".to_owned());
            }
            Action::Quit => self.should_quit = true,
            Action::OpenHelp => {
                self.help_open = true;
                self.help_scroll = 0;
            }
            Action::CloseHelp => {
                self.help_open = false;
                self.help_scroll = 0;
            }
            Action::ScrollHelpUp => self.help_scroll = self.help_scroll.saturating_sub(1),
            Action::ScrollHelpDown => self.help_scroll = self.help_scroll.saturating_add(1),
            Action::NextTab => return self.select_tab(self.active_tab.next()),
            Action::PreviousTab => return self.select_tab(self.active_tab.previous()),
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
                self.status = None;
                return vec![Effect::Background(BackgroundEffect::DownloadDescription(
                    puzzle,
                ))];
            }
            Action::RefreshCalendar if self.active_tab == Tab::Calendar => {
                self.calendar_loading = true;
                self.status = None;
                return vec![Effect::Background(BackgroundEffect::LoadCalendar {
                    year: self.selected_year,
                    refresh: true,
                })];
            }
            Action::OpenBrowser if self.active_tab == Tab::Calendar => {
                let Some(puzzle) = self.selected_puzzle_or_status() else {
                    return Vec::new();
                };
                self.status = None;
                return vec![Effect::Foreground(ForegroundEffect::OpenBrowser(puzzle))];
            }
            Action::OpenExercise if self.active_tab == Tab::Calendar => {
                if self.exercise_preparing {
                    self.status = Some("An exercise is already being prepared".to_owned());
                    return Vec::new();
                }
                if self.language_busy() {
                    self.status = Some("A language operation is already running".to_owned());
                    return Vec::new();
                }
                let Some(puzzle) = self.selected_puzzle_or_status() else {
                    return Vec::new();
                };
                self.exercise_preparing = true;
                self.status = None;
                return vec![Effect::Background(BackgroundEffect::PrepareExercise {
                    puzzle,
                    language: self.language,
                })];
            }
            Action::RunPart(part) if self.active_tab == Tab::Calendar => {
                if self.language_busy() {
                    self.status = Some("A language operation is already running".to_owned());
                    return Vec::new();
                }
                let Some(puzzle) = self.selected_puzzle_or_status() else {
                    return Vec::new();
                };
                let request = RunRequest {
                    puzzle,
                    language: self.language,
                    part,
                    input: self.run_input,
                };
                self.active_run = Some(request);
                self.run_spinner_frame = 0;
                self.run_dialog = None;
                self.status = None;
                return vec![Effect::Background(BackgroundEffect::RunSolver(request))];
            }
            Action::ToggleRunInput if self.active_tab == Tab::Calendar => {
                self.run_input = match self.run_input {
                    RunInput::Aoc => RunInput::Example,
                    RunInput::Example => RunInput::Aoc,
                };
                self.status = None;
            }
            Action::Tick if self.active_run.is_some() => {
                self.run_spinner_frame = (self.run_spinner_frame + 1) % 4;
            }
            Action::CancelRunDialog => self.run_dialog = None,
            Action::ScrollRunUp => {
                if let Some(RunDialog::Result { scroll, .. }) = &mut self.run_dialog {
                    *scroll = scroll.saturating_sub(1);
                }
            }
            Action::ScrollRunDown => {
                if let Some(RunDialog::Result { scroll, .. }) = &mut self.run_dialog {
                    *scroll = scroll.saturating_add(1);
                }
            }
            Action::SwitchLanguage if self.active_tab == Tab::Language => {
                if self.language_busy() {
                    return Vec::new();
                }
                self.language = match self.language {
                    LanguageId::Rust => LanguageId::Python,
                    LanguageId::Python => LanguageId::Rust,
                };
                self.clear_language_data();
                return self.load_language_data();
            }
            Action::RefreshLanguage if self.active_tab == Tab::Language => {
                if !self.language_busy() {
                    return self.load_language_data();
                }
            }
            Action::PreviousLanguagePane | Action::NextLanguagePane
                if self.active_tab == Tab::Language && self.language_dialog.is_none() =>
            {
                self.language_focus = match self.language_focus {
                    LanguageFocus::Packages => LanguageFocus::Libraries,
                    LanguageFocus::Libraries => LanguageFocus::Packages,
                };
            }
            Action::PreviousLanguageItem
                if self.active_tab == Tab::Language && self.language_dialog.is_none() =>
            {
                let selection = self.language_selection_mut();
                *selection = selection.saturating_sub(1);
            }
            Action::NextLanguageItem
                if self.active_tab == Tab::Language && self.language_dialog.is_none() =>
            {
                let maximum = self.language_items().len().saturating_sub(1);
                let selection = self.language_selection_mut();
                *selection = (*selection + 1).min(maximum);
            }
            Action::AddPackage if self.active_tab == Tab::Language => {
                if !self.language_busy() {
                    self.language_dialog = Some(LanguageDialog::Text {
                        kind: LanguageTextInput::AddPackage,
                        value: String::new(),
                        error: None,
                    });
                }
            }
            Action::RemoveLanguageItem if self.active_tab == Tab::Language => {
                if !self.language_busy() {
                    let action = match self.language_focus {
                        LanguageFocus::Packages => self
                            .selected_package()
                            .map(|package| LanguageConfirmation::RemovePackage(package.to_owned())),
                        LanguageFocus::Libraries => self
                            .selected_library()
                            .map(|library| LanguageConfirmation::RemoveLibrary(library.to_owned())),
                    };
                    if let Some(action) = action {
                        self.language_dialog = Some(LanguageDialog::Confirm {
                            action,
                            confirmed: false,
                        });
                    } else {
                        self.language_dialog = Some(LanguageDialog::Message(
                            "No language item is selected".to_owned(),
                        ));
                    }
                }
            }
            Action::NewLibrary if self.active_tab == Tab::Language => {
                if !self.language_busy() {
                    self.language_dialog = Some(LanguageDialog::Text {
                        kind: LanguageTextInput::Library,
                        value: String::new(),
                        error: None,
                    });
                }
            }
            Action::OpenLanguageItem if self.active_tab == Tab::Language => {
                if !self.language_busy() && self.language_focus == LanguageFocus::Libraries {
                    if let Some(library) = self.selected_library().map(str::to_owned) {
                        return self
                            .prepare_language_file(LanguageFileKind::Library(library), false);
                    }
                    self.language_dialog =
                        Some(LanguageDialog::Message("No library is selected".to_owned()));
                }
            }
            Action::OpenTemplate if self.active_tab == Tab::Language => {
                if !self.language_busy() {
                    return self.prepare_language_file(LanguageFileKind::Template, false);
                }
            }
            Action::ResetTemplate if self.active_tab == Tab::Language => {
                if !self.language_busy() {
                    self.language_dialog = Some(LanguageDialog::Confirm {
                        action: LanguageConfirmation::ResetTemplate,
                        confirmed: false,
                    });
                }
            }
            Action::RefreshConfig if self.active_tab == Tab::Config => {
                if !self.config_busy() {
                    return self.load_config();
                }
            }
            Action::PreviousConfigField
                if self.active_tab == Tab::Config && self.config_dialog.is_none() =>
            {
                self.config_selection = self.config_selection.saturating_sub(1);
            }
            Action::NextConfigField
                if self.active_tab == Tab::Config && self.config_dialog.is_none() =>
            {
                self.config_selection = (self.config_selection + 1).min(ConfigField::ALL.len() - 1);
            }
            Action::EditConfigField if self.active_tab == Tab::Config => {
                if !self.config_busy() {
                    self.open_config_editor();
                }
            }
            Action::RemoveConfigValue if self.active_tab == Tab::Config => {
                if !self.config_busy() && self.config.is_some() {
                    match self.selected_config_field() {
                        ConfigField::Year => {
                            return self.save_config(ConfigMutation::Set {
                                field: NonSecretConfigField::Year,
                                value: None,
                            });
                        }
                        ConfigField::Editor => {
                            return self.save_config(ConfigMutation::Set {
                                field: NonSecretConfigField::Editor,
                                value: None,
                            });
                        }
                        ConfigField::RunHistoryLimit => {
                            return self.save_config(ConfigMutation::Set {
                                field: NonSecretConfigField::RunHistoryLimit,
                                value: None,
                            });
                        }
                        ConfigField::Session
                            if self
                                .config
                                .as_ref()
                                .is_some_and(|config| config.session_configured) =>
                        {
                            self.config_dialog =
                                Some(ConfigDialog::ConfirmRemoveSession { confirmed: false });
                        }
                        ConfigField::Session => {}
                    }
                }
            }
            Action::ConfigInput(character) => {
                if let Some(ConfigDialog::Text { value, error, .. }) = &mut self.config_dialog {
                    value.push(character);
                    *error = None;
                }
            }
            Action::ConfigSecretInput(character) => {
                if let Some(ConfigDialog::Session { value, error }) = &mut self.config_dialog {
                    value.push(character.0);
                    *error = None;
                }
            }
            Action::ConfigBackspace => match &mut self.config_dialog {
                Some(ConfigDialog::Text { value, .. }) => {
                    value.pop();
                }
                Some(ConfigDialog::Session { value, .. }) => value.pop(),
                _ => {}
            },
            Action::ConfigToggleConfirmation => {
                if let Some(ConfigDialog::ConfirmRemoveSession { confirmed }) =
                    &mut self.config_dialog
                {
                    *confirmed = !*confirmed;
                }
            }
            Action::ConfigScrollMessageUp => {
                if let Some(ConfigDialog::Message { scroll, .. }) = &mut self.config_dialog {
                    *scroll = scroll.saturating_sub(1);
                }
            }
            Action::ConfigScrollMessageDown => {
                if let Some(ConfigDialog::Message { scroll, .. }) = &mut self.config_dialog {
                    *scroll = scroll.saturating_add(1);
                }
            }
            Action::ConfigSubmit => return self.submit_config_dialog(),
            Action::ConfigCancel => self.config_dialog = None,
            Action::DialogInput(character) => {
                if let Some(LanguageDialog::Text { value, error, .. }) = &mut self.language_dialog {
                    value.push(character);
                    *error = None;
                }
            }
            Action::DialogBackspace => {
                if let Some(LanguageDialog::Text { value, .. }) = &mut self.language_dialog {
                    value.pop();
                }
            }
            Action::DialogToggleConfirmation => {
                if let Some(LanguageDialog::Confirm { confirmed, .. }) = &mut self.language_dialog {
                    *confirmed = !*confirmed;
                }
            }
            Action::DialogSubmit => return self.submit_language_dialog(),
            Action::DialogCancel => {
                self.language_dialog = None;
                self.status = None;
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
                                effects.push(self.check_cached_description(puzzle));
                            }
                        }
                        self.status = None;
                        return effects;
                    }
                    Err(message) => self.status = Some(message),
                }
            }
            Action::CachedDescriptionFinished { puzzle, result } => {
                if Some(puzzle) != self.selected_puzzle()
                    || !matches!(
                        self.description,
                        DescriptionState::CheckingCache(checking) if checking == puzzle
                    )
                {
                    return Vec::new();
                }
                match result {
                    Ok(Some(markdown)) => {
                        self.description = DescriptionState::Loaded { puzzle, markdown };
                        self.status = None;
                    }
                    Ok(None) => {
                        self.description = DescriptionState::Empty;
                        self.status = None;
                    }
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
                        self.status = None;
                    }
                    Err(message) => {
                        self.status = Some(message.clone());
                        if !matches!(self.description, DescriptionState::Loaded { .. }) {
                            self.description = DescriptionState::Error { puzzle, message };
                        }
                    }
                }
            }
            Action::ExercisePrepared {
                puzzle,
                language,
                result,
            } => {
                self.exercise_preparing = false;
                if Some(puzzle) != self.selected_puzzle() || language != self.language {
                    return Vec::new();
                }
                match result {
                    Ok(prepared) => {
                        self.status = None;
                        return vec![Effect::Foreground(ForegroundEffect::OpenExercise(prepared))];
                    }
                    Err(message) => self.status = Some(message),
                }
            }
            Action::RunFinished { request, result } => {
                self.active_run = None;
                self.status = None;
                self.run_dialog = Some(RunDialog::Result {
                    request,
                    result,
                    scroll: 0,
                });
            }
            Action::RunEffectFailed { request, failure } => {
                self.active_run = None;
                self.status = None;
                self.run_dialog = Some(RunDialog::Result {
                    request,
                    result: Err(failure),
                    scroll: 0,
                });
            }
            Action::LanguageDataFinished { language, result } => {
                if language != self.language {
                    return Vec::new();
                }
                match result {
                    Ok(data) => {
                        self.set_language_data(data);
                        self.language_operation = LanguageOperationState::Idle;
                    }
                    Err(message) => self.show_language_error(message),
                }
            }
            Action::LanguageMutationFinished { language, result } => {
                if language != self.language {
                    return Vec::new();
                }
                match result {
                    Ok(data) => {
                        self.set_language_data(data);
                        self.language_operation = LanguageOperationState::Idle;
                    }
                    Err(message) => self.show_language_error(message),
                }
            }
            Action::LanguageFilePrepared { language, result } => {
                if language != self.language {
                    return Vec::new();
                }
                match result {
                    Ok(prepared) => {
                        self.language_file_opening = Some(prepared.kind.clone());
                        self.language_operation = match prepared.kind {
                            LanguageFileKind::Library(_) => LanguageOperationState::Running {
                                packages: None,
                                libraries: Some("opening...".to_owned()),
                            },
                            LanguageFileKind::Template => LanguageOperationState::Running {
                                packages: None,
                                libraries: None,
                            },
                        };
                        return vec![Effect::Foreground(ForegroundEffect::OpenLanguageFile(
                            prepared,
                        ))];
                    }
                    Err(message) => self.show_language_error(message),
                }
            }
            Action::LanguageEffectFailed(message) => {
                self.language_file_opening = None;
                self.show_language_error(message);
            }
            Action::ConfigLoaded { result } => match result {
                Ok(config) => {
                    self.config = Some(config);
                    self.config_operation = ConfigOperationState::Idle;
                    self.config_dialog = None;
                    self.status = None;
                }
                Err(message) => self.show_config_error(message),
            },
            Action::ConfigSaved { result } => match result {
                Ok(config) => {
                    self.config = Some(config);
                    self.config_operation = ConfigOperationState::Idle;
                    self.config_dialog = None;
                    self.status = None;
                    if self.quit_after_config_save {
                        self.should_quit = true;
                    }
                    self.quit_after_config_save = false;
                }
                Err(message) => {
                    self.quit_after_config_save = false;
                    self.show_config_error(message);
                }
            },
            Action::ConfigEffectFailed(message) => {
                self.quit_after_config_save = false;
                self.show_config_error(message);
            }
            Action::ForegroundFinished(result) => {
                if let Some(kind) = self.language_file_opening.take() {
                    match result {
                        Ok(()) => match kind {
                            LanguageFileKind::Library(_) => {
                                return self.load_language_data_with_activity(
                                    None,
                                    Some("loading...".to_owned()),
                                );
                            }
                            LanguageFileKind::Template => {
                                self.language_operation = LanguageOperationState::Idle;
                            }
                        },
                        Err(message) => self.show_language_error(message),
                    }
                } else {
                    self.status = result.err();
                    if self.status.is_none()
                        && self.active_tab == Tab::Language
                        && !self.language_loaded
                    {
                        return self.load_language_data();
                    }
                }
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

    fn select_tab(&mut self, tab: Tab) -> Vec<Effect> {
        self.active_tab = tab;
        if self.help_open {
            self.help_scroll = 0;
        }
        if tab == Tab::Language && !self.language_loaded && !self.language_busy() {
            return self.load_language_data();
        }
        if tab == Tab::Config && self.config.is_none() && !self.config_busy() {
            return self.load_config();
        }
        Vec::new()
    }

    fn config_busy(&self) -> bool {
        matches!(self.config_operation, ConfigOperationState::Running(_))
    }

    fn config_saving(&self) -> bool {
        self.config_operation == ConfigOperationState::Running("saving...")
    }

    fn load_config(&mut self) -> Vec<Effect> {
        self.config_operation = ConfigOperationState::Running("loading...");
        vec![Effect::Background(BackgroundEffect::LoadConfig {
            latest_year: self.latest_puzzle.year,
        })]
    }

    fn selected_config_field(&self) -> ConfigField {
        ConfigField::ALL[self.config_selection]
    }

    fn open_config_editor(&mut self) {
        if self.config.is_none() {
            return;
        }
        let field = self.selected_config_field();
        self.config_dialog = Some(match field {
            ConfigField::Year => ConfigDialog::Text {
                field: NonSecretConfigField::Year,
                value: String::new(),
                error: None,
            },
            ConfigField::Editor => ConfigDialog::Text {
                field: NonSecretConfigField::Editor,
                value: String::new(),
                error: None,
            },
            ConfigField::RunHistoryLimit => ConfigDialog::Text {
                field: NonSecretConfigField::RunHistoryLimit,
                value: String::new(),
                error: None,
            },
            ConfigField::Session => ConfigDialog::Session {
                value: SecretString::empty(),
                error: None,
            },
        });
    }

    fn submit_config_dialog(&mut self) -> Vec<Effect> {
        let Some(dialog) = self.config_dialog.take() else {
            return Vec::new();
        };
        match dialog {
            ConfigDialog::Text {
                field,
                value,
                error: _,
            } => {
                let value = value.trim().to_owned();
                let mutation = match field {
                    NonSecretConfigField::Year if value.is_empty() => {
                        ConfigMutation::Set { field, value: None }
                    }
                    NonSecretConfigField::Year => match value.parse::<PuzzleYear>() {
                        Ok(year) if year <= self.latest_puzzle.year => ConfigMutation::Set {
                            field,
                            value: Some(year.to_string()),
                        },
                        _ => {
                            self.config_dialog = Some(ConfigDialog::Text {
                                field,
                                value,
                                error: Some(format!(
                                    "Enter a released year from {} through {}",
                                    PuzzleYear::MIN,
                                    self.latest_puzzle.year
                                )),
                            });
                            return Vec::new();
                        }
                    },
                    NonSecretConfigField::Editor => ConfigMutation::Set {
                        field,
                        value: (!value.is_empty()).then_some(value),
                    },
                    NonSecretConfigField::RunHistoryLimit if value.is_empty() => {
                        ConfigMutation::Set { field, value: None }
                    }
                    NonSecretConfigField::RunHistoryLimit => {
                        if value.parse::<RunHistoryLimit>().is_err() {
                            self.config_dialog = Some(ConfigDialog::Text {
                                field,
                                value,
                                error: Some("Enter a positive integer".to_owned()),
                            });
                            return Vec::new();
                        }
                        ConfigMutation::Set {
                            field,
                            value: Some(value),
                        }
                    }
                };
                self.save_config(mutation)
            }
            ConfigDialog::Session { value, error: _ } => {
                let Some(value) = value.trimmed() else {
                    self.config_dialog = Some(ConfigDialog::Session {
                        value: SecretString::empty(),
                        error: Some("Session cannot be empty".to_owned()),
                    });
                    return Vec::new();
                };
                self.save_config(ConfigMutation::SetSession(value))
            }
            ConfigDialog::ConfirmRemoveSession { confirmed } => {
                if confirmed {
                    self.save_config(ConfigMutation::RemoveSession)
                } else {
                    Vec::new()
                }
            }
            ConfigDialog::Message { .. } => Vec::new(),
        }
    }

    fn save_config(&mut self, mutation: ConfigMutation) -> Vec<Effect> {
        self.config_operation = ConfigOperationState::Running("saving...");
        vec![Effect::Background(BackgroundEffect::MutateConfig {
            latest_year: self.latest_puzzle.year,
            mutation,
        })]
    }

    fn show_config_error(&mut self, message: String) {
        self.config_operation = ConfigOperationState::Idle;
        self.config_dialog = Some(ConfigDialog::Message { message, scroll: 0 });
    }

    fn language_busy(&self) -> bool {
        matches!(
            self.language_operation,
            LanguageOperationState::Running { .. }
        ) || self.language_file_opening.is_some()
            || self.exercise_preparing
            || self.active_run.is_some()
    }

    fn clear_language_data(&mut self) {
        self.language_packages.clear();
        self.language_libraries.clear();
        self.language_package_selection = 0;
        self.language_library_selection = 0;
        self.language_loaded = false;
    }

    fn set_language_data(&mut self, data: LanguageData) {
        self.language_packages = data.packages;
        self.language_libraries = data.libraries;
        self.language_package_selection = self
            .language_package_selection
            .min(self.language_packages.len().saturating_sub(1));
        self.language_library_selection = self
            .language_library_selection
            .min(self.language_libraries.len().saturating_sub(1));
        self.language_loaded = true;
        self.status = None;
    }

    fn load_language_data(&mut self) -> Vec<Effect> {
        self.load_language_data_with_activity(
            Some("loading...".to_owned()),
            Some("loading...".to_owned()),
        )
    }

    fn load_language_data_with_activity(
        &mut self,
        packages: Option<String>,
        libraries: Option<String>,
    ) -> Vec<Effect> {
        self.status = None;
        self.language_operation = LanguageOperationState::Running {
            packages,
            libraries,
        };
        vec![Effect::Background(BackgroundEffect::LoadLanguageData {
            language: self.language,
        })]
    }

    fn mutate_language(&mut self, mutation: LanguageMutation) -> Vec<Effect> {
        self.status = None;
        self.language_operation = match &mutation {
            LanguageMutation::AddPackage(_) => LanguageOperationState::Running {
                packages: Some("adding...".to_owned()),
                libraries: None,
            },
            LanguageMutation::RemovePackage(_) => LanguageOperationState::Running {
                packages: Some("removing...".to_owned()),
                libraries: None,
            },
            LanguageMutation::RemoveLibrary(_) => LanguageOperationState::Running {
                packages: None,
                libraries: Some("removing...".to_owned()),
            },
        };
        vec![Effect::Background(BackgroundEffect::MutateLanguage {
            language: self.language,
            mutation,
        })]
    }

    fn prepare_language_file(&mut self, kind: LanguageFileKind, reset: bool) -> Vec<Effect> {
        self.status = None;
        self.language_operation = match &kind {
            LanguageFileKind::Library(_) => LanguageOperationState::Running {
                packages: None,
                libraries: Some("opening...".to_owned()),
            },
            LanguageFileKind::Template => LanguageOperationState::Running {
                packages: None,
                libraries: None,
            },
        };
        vec![Effect::Background(BackgroundEffect::PrepareLanguageFile {
            language: self.language,
            kind,
            reset,
        })]
    }

    fn submit_language_dialog(&mut self) -> Vec<Effect> {
        let Some(dialog) = self.language_dialog.take() else {
            return Vec::new();
        };
        match dialog {
            LanguageDialog::Text {
                kind,
                value,
                error: _,
            } => {
                let value = value.trim().to_owned();
                if value.is_empty() {
                    let error = match kind {
                        LanguageTextInput::AddPackage => "Package name cannot be empty",
                        LanguageTextInput::Library => "Library name cannot be empty",
                    };
                    self.language_dialog = Some(LanguageDialog::Text {
                        kind,
                        value,
                        error: Some(error.to_owned()),
                    });
                    return Vec::new();
                }
                match kind {
                    LanguageTextInput::AddPackage => {
                        self.mutate_language(LanguageMutation::AddPackage(value))
                    }
                    LanguageTextInput::Library => {
                        self.prepare_language_file(LanguageFileKind::Library(value), false)
                    }
                }
            }
            LanguageDialog::Confirm { action, confirmed } => {
                if !confirmed {
                    self.status = None;
                    return Vec::new();
                }
                match action {
                    LanguageConfirmation::RemovePackage(package) => {
                        self.mutate_language(LanguageMutation::RemovePackage(package))
                    }
                    LanguageConfirmation::RemoveLibrary(library) => {
                        self.mutate_language(LanguageMutation::RemoveLibrary(library))
                    }
                    LanguageConfirmation::ResetTemplate => {
                        self.prepare_language_file(LanguageFileKind::Template, true)
                    }
                }
            }
            LanguageDialog::Message(_) => Vec::new(),
        }
    }

    fn show_language_error(&mut self, message: String) {
        self.language_operation = LanguageOperationState::Idle;
        self.status = None;
        self.language_dialog = Some(LanguageDialog::Message(message));
    }

    fn language_items(&self) -> &[String] {
        match self.language_focus {
            LanguageFocus::Packages => &self.language_packages,
            LanguageFocus::Libraries => &self.language_libraries,
        }
    }

    fn language_selection_mut(&mut self) -> &mut usize {
        match self.language_focus {
            LanguageFocus::Packages => &mut self.language_package_selection,
            LanguageFocus::Libraries => &mut self.language_library_selection,
        }
    }

    fn selected_package(&self) -> Option<&str> {
        self.language_packages
            .get(self.language_package_selection)
            .map(String::as_str)
    }

    fn selected_library(&self) -> Option<&str> {
        self.language_libraries
            .get(self.language_library_selection)
            .map(String::as_str)
    }

    fn select_year(&mut self, year: PuzzleYear) -> Vec<Effect> {
        self.selected_year = year;
        self.selected_puzzle = None;
        self.calendar = None;
        self.calendar_loading = true;
        self.calendar_scroll = (0, 0);
        self.clear_description();
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
        vec![self.check_cached_description(puzzle)]
    }

    fn selected_puzzle_or_status(&mut self) -> Option<PuzzleId> {
        if self.selected_puzzle.is_none() {
            self.status = Some("No calendar puzzle is selected".to_owned());
        }
        self.selected_puzzle
    }

    fn check_cached_description(&mut self, puzzle: PuzzleId) -> Effect {
        self.description = DescriptionState::CheckingCache(puzzle);
        self.description_scroll = 0;
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
    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear};

    use super::{
        Action, App, BackgroundEffect, ConfigData, ConfigDialog, ConfigField, ConfigMutation,
        ConfigOperationState, DescriptionState, Effect, ForegroundEffect, LanguageData,
        LanguageDialog, LanguageFileKind, LanguageMutation, LanguageOperationState,
        NonSecretConfigField, PreparedExercise, RunDialog, RunFailure, RunInput, RunPartReport,
        RunReport,
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

    fn language_app() -> App {
        let mut app = app();
        app.update(Action::NextTab);
        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Ok(LanguageData {
                packages: vec!["anyhow".to_owned()],
                libraries: vec!["grid".to_owned()],
            }),
        });
        app
    }

    fn config_app(session_configured: bool) -> App {
        let mut app = app();
        app.update(Action::PreviousTab);
        app.update(Action::ConfigLoaded {
            result: Ok(ConfigData {
                year: "2026".to_owned(),
                editor: Some("vim".to_owned()),
                run_history_limit: "10".to_owned(),
                session_configured,
            }),
        });
        app
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
            app.description,
            DescriptionState::CheckingCache(puzzle(7, 2026))
        );
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
        assert_eq!(
            app.description,
            DescriptionState::CheckingCache(puzzle(3, 2026))
        );
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

        assert_eq!(
            app.description,
            DescriptionState::CheckingCache(puzzle(9, 2026))
        );
        assert!(!app.description_downloading(stale));
        assert_eq!(app.selected_puzzle(), Some(puzzle(9, 2026)));
        assert!(app.status.is_none());
    }

    #[test]
    fn cache_misses_are_silent() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();
        let status = app.status.clone();
        assert_eq!(app.description, DescriptionState::CheckingCache(puzzle));

        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(None),
        });
        assert_eq!(app.description, DescriptionState::Empty);
        assert_eq!(app.status, status);
    }

    #[test]
    fn cache_errors_are_visible() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();

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
    fn late_cache_results_do_not_replace_a_completed_download() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();
        app.update(Action::DownloadDescription);
        app.update(Action::DescriptionDownloaded {
            puzzle,
            result: Ok("downloaded preview".to_owned()),
        });

        app.update(Action::CachedDescriptionFinished {
            puzzle,
            result: Ok(Some("older cached preview".to_owned())),
        });

        assert_eq!(
            app.description,
            DescriptionState::Loaded {
                puzzle,
                markdown: "downloaded preview".to_owned()
            }
        );
    }

    #[test]
    fn stale_cache_results_do_not_replace_the_selected_puzzle() {
        let mut app = selected_app();
        let stale = app.selected_puzzle().unwrap();
        app.update(Action::NextCalendarPuzzle);
        let selected = app.selected_puzzle().unwrap();

        app.update(Action::CachedDescriptionFinished {
            puzzle: stale,
            result: Ok(Some("stale preview".to_owned())),
        });

        assert_eq!(app.description, DescriptionState::CheckingCache(selected));
    }

    #[test]
    fn routine_operations_clear_footer_status() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();

        app.status = Some("previous problem".to_owned());
        app.update(Action::DownloadDescription);
        assert!(app.status.is_none());
        app.update(Action::DescriptionDownloaded {
            puzzle,
            result: Ok("downloaded description".to_owned()),
        });
        assert!(app.status.is_none());

        app.status = Some("previous problem".to_owned());
        app.update(Action::RefreshCalendar);
        assert!(app.status.is_none());
        load_calendar(&mut app, calendar(vec![vec![Some(puzzle)]]), true);
        assert!(app.status.is_none());

        app.status = Some("previous problem".to_owned());
        assert_eq!(
            app.update(Action::OpenBrowser),
            vec![Effect::Foreground(ForegroundEffect::OpenBrowser(puzzle))]
        );
        assert!(app.status.is_none());
        app.update(Action::ForegroundFinished(Ok(())));
        assert!(app.status.is_none());
    }

    #[test]
    fn foreground_failures_remain_visible() {
        let mut app = selected_app();

        app.update(Action::ForegroundFinished(Err("browser failed".to_owned())));

        assert_eq!(app.status.as_deref(), Some("browser failed"));
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
    fn solver_run_dispatches_once_and_blocks_language_work_and_quit() {
        let mut app = selected_app();
        let selected = app.selected_puzzle().unwrap();

        let effects = app.update(Action::RunPart(PuzzlePart::One));
        let request = super::RunRequest {
            puzzle: selected,
            language: LanguageId::Rust,
            part: PuzzlePart::One,
            input: RunInput::Aoc,
        };
        assert_eq!(
            effects,
            vec![Effect::Background(BackgroundEffect::RunSolver(request))]
        );
        assert_eq!(app.active_run, Some(request));
        assert!(app.status.is_none());

        assert!(app.update(Action::RunPart(PuzzlePart::Two)).is_empty());
        assert!(app.update(Action::OpenExercise).is_empty());
        app.update(Action::NextTab);
        assert_eq!(app.active_tab, super::Tab::Calendar);
        assert!(app.update(Action::SwitchLanguage).is_empty());
        app.update(Action::Tick);
        assert_eq!(app.run_spinner_frame, 1);
        app.update(Action::Quit);
        assert!(!app.should_quit);
        assert_eq!(
            app.status.as_deref(),
            Some("Wait for the solver run to finish")
        );
    }

    #[test]
    fn completed_run_keeps_results_and_persistence_warning_tagged_to_request() {
        let mut app = selected_app();
        app.update(Action::ToggleRunInput);
        assert_eq!(app.run_input, RunInput::Example);
        assert!(app.status.is_none());
        app.update(Action::RunPart(PuzzlePart::One));
        let request = app.active_run.unwrap();
        let report = RunReport {
            request,
            compile_stdout: String::new(),
            compile_stderr: String::new(),
            solver_stdout: "solver output".to_owned(),
            solver_stderr: String::new(),
            parts: vec![RunPartReport {
                part: PuzzlePart::One,
                answer: "42".to_owned(),
                runtime_ms: 7,
            }],
            warning: Some("Run completed, but timing persistence failed".to_owned()),
        };

        app.update(Action::RunFinished {
            request,
            result: Ok(report.clone()),
        });

        assert!(app.active_run.is_none());
        assert_eq!(
            app.run_dialog,
            Some(RunDialog::Result {
                request,
                result: Ok(report),
                scroll: 0,
            })
        );
    }

    #[test]
    fn queue_failure_releases_active_run_and_keeps_request_context() {
        let mut app = selected_app();
        app.update(Action::RunPart(PuzzlePart::Two));
        let request = app.active_run.unwrap();
        let failure = RunFailure {
            request,
            summary: "Could not queue solver run".to_owned(),
            details: Some("worker stopped".to_owned()),
        };

        app.update(Action::RunEffectFailed {
            request,
            failure: failure.clone(),
        });

        assert!(app.active_run.is_none());
        assert_eq!(
            app.run_dialog,
            Some(RunDialog::Result {
                request,
                result: Err(failure),
                scroll: 0,
            })
        );
    }

    #[test]
    fn exercise_is_prepared_before_foreground_editor_handoff() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();

        assert_eq!(
            app.update(Action::OpenExercise),
            vec![Effect::Background(BackgroundEffect::PrepareExercise {
                puzzle,
                language: LanguageId::Rust,
            })]
        );
        assert!(app.exercise_preparing);
        assert!(app.status.is_none());
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
                language: LanguageId::Rust,
                result: Ok(prepared.clone()),
            }),
            vec![Effect::Foreground(ForegroundEffect::OpenExercise(prepared))]
        );
        assert!(!app.exercise_preparing);
        assert!(app.status.is_none());
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
                language: LanguageId::Rust,
                result: Err("stale preparation".to_owned()),
            })
            .is_empty());
        assert!(!app.exercise_preparing);
        assert_eq!(app.update(Action::OpenExercise).len(), 1);
    }

    #[test]
    fn pending_exercise_preparation_blocks_other_language_jobs() {
        let mut app = selected_app();
        let puzzle = app.selected_puzzle().unwrap();
        app.update(Action::OpenExercise);
        app.update(Action::NextTab);

        assert!(app.update(Action::SwitchLanguage).is_empty());
        assert!(app.update(Action::RefreshLanguage).is_empty());
        app.update(Action::AddPackage);
        assert_eq!(app.language, LanguageId::Rust);
        assert!(app.language_dialog.is_none());

        app.update(Action::ExercisePrepared {
            puzzle,
            language: LanguageId::Rust,
            result: Err("preparation failed".to_owned()),
        });
        assert!(!app.exercise_preparing);
    }

    #[test]
    fn active_language_job_blocks_calendar_exercise_preparation() {
        let mut app = language_app();
        app.update(Action::RemoveLanguageItem);
        app.update(Action::DialogToggleConfirmation);
        app.update(Action::DialogSubmit);
        app.update(Action::NextTab);
        app.update(Action::NextTab);
        load_calendar(
            &mut app,
            calendar(vec![vec![Some(puzzle(10, 2026))]]),
            false,
        );

        assert!(app.update(Action::OpenExercise).is_empty());
        assert_eq!(
            app.status.as_deref(),
            Some("A language operation is already running")
        );
    }

    #[test]
    fn entering_language_loads_read_only_lists_once() {
        let mut app = app();

        assert_eq!(
            app.update(Action::NextTab),
            vec![Effect::Background(BackgroundEffect::LoadLanguageData {
                language: LanguageId::Rust,
            })]
        );
        assert_eq!(app.active_tab, super::Tab::Language);
        assert_eq!(
            app.language_operation,
            LanguageOperationState::Running {
                packages: Some("loading...".to_owned()),
                libraries: Some("loading...".to_owned()),
            }
        );

        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Ok(LanguageData {
                packages: vec![],
                libraries: vec![],
            }),
        });
        app.update(Action::NextTab);
        assert!(app.update(Action::PreviousTab).is_empty());
    }

    #[test]
    fn session_language_controls_calendar_exercise_preparation() {
        let mut app = language_app();
        app.update(Action::SwitchLanguage);
        app.update(Action::LanguageDataFinished {
            language: LanguageId::Python,
            result: Ok(LanguageData {
                packages: vec![],
                libraries: vec![],
            }),
        });
        app.update(Action::NextTab);
        app.update(Action::NextTab);
        load_calendar(
            &mut app,
            calendar(vec![vec![Some(puzzle(10, 2026))]]),
            false,
        );

        assert_eq!(
            app.update(Action::OpenExercise),
            vec![Effect::Background(BackgroundEffect::PrepareExercise {
                puzzle: puzzle(10, 2026),
                language: LanguageId::Python,
            })]
        );
    }

    #[test]
    fn destructive_language_dialogs_cancel_by_default() {
        let mut app = language_app();

        app.update(Action::RemoveLanguageItem);
        assert!(matches!(
            app.language_dialog,
            Some(LanguageDialog::Confirm {
                confirmed: false,
                ..
            })
        ));

        assert!(app.update(Action::DialogSubmit).is_empty());
        assert!(app.language_dialog.is_none());
        assert_eq!(app.language_packages, vec!["anyhow"]);
    }

    #[test]
    fn confirmed_package_removal_dispatches_one_serialized_job() {
        let mut app = language_app();
        app.update(Action::RemoveLanguageItem);
        app.update(Action::DialogToggleConfirmation);

        assert_eq!(
            app.update(Action::DialogSubmit),
            vec![Effect::Background(BackgroundEffect::MutateLanguage {
                language: LanguageId::Rust,
                mutation: LanguageMutation::RemovePackage("anyhow".to_owned()),
            })]
        );
        assert!(app.update(Action::RefreshLanguage).is_empty());
    }

    #[test]
    fn library_name_dialog_prepares_the_editor_without_creating_a_file_in_the_reducer() {
        let mut app = language_app();
        app.update(Action::NewLibrary);
        app.update(Action::DialogInput('m'));
        app.update(Action::DialogInput('a'));
        app.update(Action::DialogInput('t'));
        app.update(Action::DialogInput('h'));

        assert_eq!(
            app.update(Action::DialogSubmit),
            vec![Effect::Background(BackgroundEffect::PrepareLanguageFile {
                language: LanguageId::Rust,
                kind: LanguageFileKind::Library("math".to_owned()),
                reset: false,
            })]
        );
    }

    #[test]
    fn stale_language_results_do_not_replace_the_current_selection() {
        let mut app = language_app();
        app.update(Action::SwitchLanguage);

        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Ok(LanguageData {
                packages: vec!["stale".to_owned()],
                libraries: vec![],
            }),
        });

        assert_eq!(app.language, LanguageId::Python);
        assert!(app.language_packages.is_empty());
        assert!(matches!(
            app.language_operation,
            LanguageOperationState::Running { .. }
        ));
    }

    #[test]
    fn language_errors_are_dismissible_messages() {
        let mut app = app();
        app.update(Action::NextTab);

        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Err("package query failed".to_owned()),
        });

        assert_eq!(
            app.language_dialog,
            Some(LanguageDialog::Message("package query failed".to_owned()))
        );
        assert_eq!(app.language_operation, LanguageOperationState::Idle);
        assert!(app.status.is_none());
        app.update(Action::DialogCancel);
        assert!(app.language_dialog.is_none());
    }

    #[test]
    fn library_editor_refresh_marks_only_the_library_pane() {
        let mut app = language_app();
        app.language_file_opening = Some(LanguageFileKind::Library("grid".to_owned()));
        app.language_operation = LanguageOperationState::Running {
            packages: None,
            libraries: Some("opening...".to_owned()),
        };

        assert_eq!(
            app.update(Action::ForegroundFinished(Ok(()))),
            vec![Effect::Background(BackgroundEffect::LoadLanguageData {
                language: LanguageId::Rust,
            })]
        );
        assert_eq!(
            app.language_operation,
            LanguageOperationState::Running {
                packages: None,
                libraries: Some("loading...".to_owned()),
            }
        );
    }

    #[test]
    fn entering_config_loads_fields_lazily() {
        let mut app = app();

        assert_eq!(
            app.update(Action::PreviousTab),
            vec![Effect::Background(BackgroundEffect::LoadConfig {
                latest_year: PuzzleYear::new(2026).unwrap(),
            })]
        );
        assert_eq!(app.active_tab, super::Tab::Config);
        assert_eq!(
            app.config_operation,
            ConfigOperationState::Running("loading...")
        );
        assert_eq!(ConfigField::ALL.len(), 4);
        assert!(!ConfigField::ALL
            .iter()
            .any(|field| field.label().contains("Language")));
    }

    #[test]
    fn config_editors_start_with_an_empty_input() {
        for selection in 0..ConfigField::ALL.len() {
            let mut app = config_app(false);
            app.config_selection = selection;

            app.update(Action::EditConfigField);

            match app.config_dialog {
                Some(ConfigDialog::Text { value, .. }) => assert!(value.is_empty()),
                Some(ConfigDialog::Session { value, .. }) => assert!(value.is_empty()),
                dialog => panic!("unexpected config dialog: {dialog:?}"),
            }
        }
    }

    #[test]
    fn config_year_is_released_and_does_not_move_the_calendar() {
        let mut app = config_app(false);
        let selected_year = app.selected_year;
        app.config_dialog = Some(ConfigDialog::Text {
            field: NonSecretConfigField::Year,
            value: "2027".to_owned(),
            error: None,
        });

        assert!(app.update(Action::ConfigSubmit).is_empty());
        assert!(matches!(
            app.config_dialog,
            Some(ConfigDialog::Text { error: Some(_), .. })
        ));

        app.config_dialog = Some(ConfigDialog::Text {
            field: NonSecretConfigField::Year,
            value: "2025".to_owned(),
            error: None,
        });
        assert_eq!(
            app.update(Action::ConfigSubmit),
            vec![Effect::Background(BackgroundEffect::MutateConfig {
                latest_year: PuzzleYear::new(2026).unwrap(),
                mutation: ConfigMutation::Set {
                    field: NonSecretConfigField::Year,
                    value: Some("2025".to_owned()),
                },
            })]
        );
        assert_eq!(app.selected_year, selected_year);
    }

    #[test]
    fn blank_non_secret_config_values_reset_the_override() {
        let mut app = config_app(false);
        app.config_dialog = Some(ConfigDialog::Text {
            field: NonSecretConfigField::Year,
            value: String::new(),
            error: None,
        });

        assert_eq!(
            app.update(Action::ConfigSubmit),
            vec![Effect::Background(BackgroundEffect::MutateConfig {
                latest_year: PuzzleYear::new(2026).unwrap(),
                mutation: ConfigMutation::Set {
                    field: NonSecretConfigField::Year,
                    value: None,
                },
            })]
        );
    }

    #[test]
    fn reset_key_resets_each_non_secret_config_field() {
        for (selection, field) in [
            (0, NonSecretConfigField::Year),
            (1, NonSecretConfigField::Editor),
            (2, NonSecretConfigField::RunHistoryLimit),
        ] {
            let mut app = config_app(false);
            app.config_selection = selection;

            assert_eq!(
                app.update(Action::RemoveConfigValue),
                vec![Effect::Background(BackgroundEffect::MutateConfig {
                    latest_year: PuzzleYear::new(2026).unwrap(),
                    mutation: ConfigMutation::Set { field, value: None },
                })]
            );
        }
    }

    #[test]
    fn config_editor_and_run_history_use_domain_validation() {
        let mut app = config_app(false);
        app.config_dialog = Some(ConfigDialog::Text {
            field: NonSecretConfigField::RunHistoryLimit,
            value: "0".to_owned(),
            error: None,
        });
        assert!(app.update(Action::ConfigSubmit).is_empty());
        assert!(matches!(
            app.config_dialog,
            Some(ConfigDialog::Text { error: Some(_), .. })
        ));

        app.config_dialog = Some(ConfigDialog::Text {
            field: NonSecretConfigField::Editor,
            value: "  code --wait  ".to_owned(),
            error: None,
        });
        assert!(matches!(
            app.update(Action::ConfigSubmit).as_slice(),
            [Effect::Background(BackgroundEffect::MutateConfig {
                mutation: ConfigMutation::Set {
                    field: NonSecretConfigField::Editor,
                    value: Some(value),
                },
                ..
            })] if value == "code --wait"
        ));
    }

    #[test]
    fn session_input_is_redacted_and_blank_input_is_rejected() {
        let mut app = config_app(false);
        app.config_selection = 3;
        app.update(Action::EditConfigField);
        assert!(matches!(
            app.config_dialog,
            Some(ConfigDialog::Session { .. })
        ));

        assert!(app.update(Action::ConfigSubmit).is_empty());
        assert!(matches!(
            app.config_dialog,
            Some(ConfigDialog::Session { error: Some(_), .. })
        ));

        for character in "sensitive-value".chars() {
            app.update(Action::ConfigSecretInput(super::SecretCharacter(character)));
        }
        let debug = format!("{:?}", app.config_dialog);
        assert!(!debug.contains("sensitive-value"));
        let effects = app.update(Action::ConfigSubmit);
        assert_eq!(effects.len(), 1);
        assert!(!format!("{effects:?}").contains("sensitive-value"));
        assert!(matches!(
            &effects[0],
            Effect::Background(BackgroundEffect::MutateConfig {
                mutation: ConfigMutation::SetSession(_),
                ..
            })
        ));
    }

    #[test]
    fn session_removal_confirmation_defaults_to_cancel() {
        let mut app = config_app(true);
        app.config_selection = 3;
        app.update(Action::RemoveConfigValue);
        assert_eq!(
            app.config_dialog,
            Some(ConfigDialog::ConfirmRemoveSession { confirmed: false })
        );
        assert!(app.update(Action::ConfigSubmit).is_empty());

        app.update(Action::RemoveConfigValue);
        app.update(Action::ConfigToggleConfirmation);
        assert!(matches!(
            app.update(Action::ConfigSubmit).as_slice(),
            [Effect::Background(BackgroundEffect::MutateConfig {
                mutation: ConfigMutation::RemoveSession,
                ..
            })]
        ));
    }

    #[test]
    fn config_failures_clear_progress_and_open_a_message() {
        let mut app = app();
        app.update(Action::PreviousTab);

        app.update(Action::ConfigLoaded {
            result: Err("config read failed".to_owned()),
        });

        assert_eq!(app.config_operation, ConfigOperationState::Idle);
        assert_eq!(
            app.config_dialog,
            Some(ConfigDialog::Message {
                message: "config read failed".to_owned(),
                scroll: 0,
            })
        );
    }

    #[test]
    fn quit_waits_for_a_confirmed_config_save() {
        let mut app = config_app(false);
        app.config_dialog = Some(ConfigDialog::Text {
            field: NonSecretConfigField::Editor,
            value: "vim".to_owned(),
            error: None,
        });
        app.update(Action::ConfigSubmit);

        app.update(Action::Quit);
        assert!(!app.should_quit);
        assert_eq!(
            app.status.as_deref(),
            Some("Wait for the configuration save to finish")
        );

        app.update(Action::ConfigSaved {
            result: Ok(ConfigData {
                year: "2026".to_owned(),
                editor: Some("vim".to_owned()),
                run_history_limit: "10".to_owned(),
                session_configured: false,
            }),
        });
        assert!(app.should_quit);
        assert!(app.status.is_none());
    }

    #[test]
    fn failed_config_save_cancels_deferred_quit() {
        let mut app = config_app(false);
        app.config_dialog = Some(ConfigDialog::Text {
            field: NonSecretConfigField::Editor,
            value: "vim".to_owned(),
            error: None,
        });
        app.update(Action::ConfigSubmit);
        app.update(Action::Quit);

        app.update(Action::ConfigSaved {
            result: Err("save failed".to_owned()),
        });

        assert!(!app.should_quit);
        assert!(!app.quit_after_config_save);
        assert_eq!(
            app.config_dialog,
            Some(ConfigDialog::Message {
                message: "save failed".to_owned(),
                scroll: 0,
            })
        );
    }
}
