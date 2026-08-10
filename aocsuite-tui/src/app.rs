use std::{collections::HashSet, path::PathBuf};

use aocsuite_parser::{AocSubmissionResult, Calendar};
use aocsuite_utils::{LanguageId, PuzzleId, PuzzlePart, PuzzleYear, RunHistoryLimit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RunInput {
    Aoc,
    Example,
}

pub(crate) fn friendly_puzzle(puzzle: PuzzleId) -> String {
    format!("{} Day {}", puzzle.year, puzzle.day)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunRequest {
    pub puzzle: PuzzleId,
    pub language: LanguageId,
    pub part: PuzzlePart,
    pub input: RunInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunPartReport {
    pub part: PuzzlePart,
    pub answer: String,
    pub runtime_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunReport {
    pub compile_stdout: String,
    pub compile_stderr: String,
    pub solver_stdout: String,
    pub solver_stderr: String,
    pub parts: Vec<RunPartReport>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunFailure {
    pub summary: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunDialog {
    pub request: RunRequest,
    pub result: Result<RunReport, RunFailure>,
    pub scroll: u16,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SubmissionRequest {
    pub puzzle: PuzzleId,
    pub part: PuzzlePart,
    answer: String,
}

impl SubmissionRequest {
    pub(crate) fn new(puzzle: PuzzleId, part: PuzzlePart, answer: String) -> Self {
        Self {
            puzzle,
            part,
            answer,
        }
    }

    pub(crate) fn answer(&self) -> &str {
        &self.answer
    }
}

impl std::fmt::Debug for SubmissionRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SubmissionRequest")
            .field("puzzle", &self.puzzle)
            .field("part", &self.part)
            .field("answer", &"[REDACTED]")
            .finish()
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum SubmissionDialog {
    Part {
        puzzle: PuzzleId,
        part: PuzzlePart,
    },
    Answer {
        puzzle: PuzzleId,
        part: PuzzlePart,
        answer: String,
        error: Option<String>,
    },
    Confirm {
        request: SubmissionRequest,
        submit: bool,
    },
    Outcome {
        puzzle: PuzzleId,
        part: PuzzlePart,
        result: Result<AocSubmissionResult, String>,
        scroll: u16,
    },
}

impl std::fmt::Debug for SubmissionDialog {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Part { puzzle, part } => formatter
                .debug_struct("Part")
                .field("puzzle", puzzle)
                .field("part", part)
                .finish(),
            Self::Answer {
                puzzle,
                part,
                error,
                ..
            } => formatter
                .debug_struct("Answer")
                .field("puzzle", puzzle)
                .field("part", part)
                .field("answer", &"[REDACTED]")
                .field("error", error)
                .finish(),
            Self::Confirm { request, submit } => formatter
                .debug_struct("Confirm")
                .field("request", request)
                .field("submit", submit)
                .finish(),
            Self::Outcome {
                puzzle,
                part,
                result,
                scroll,
            } => formatter
                .debug_struct("Outcome")
                .field("puzzle", puzzle)
                .field("part", part)
                .field("result", result)
                .field("scroll", scroll)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tab {
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
pub(crate) enum DescriptionState {
    CheckingCache(PuzzleId),
    Empty,
    Loaded { puzzle: PuzzleId, markdown: String },
    Error { puzzle: PuzzleId, message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LanguageFocus {
    Packages,
    Libraries,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LanguageOperationState {
    Idle,
    Running {
        packages: Option<String>,
        libraries: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LanguageTextInput {
    AddPackage,
    Library,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LanguageConfirmation {
    RemovePackage(String),
    RemoveLibrary(String),
    ResetTemplate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LanguageDialog {
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
pub(crate) struct LanguageData {
    pub packages: Vec<String>,
    pub libraries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LanguageMutation {
    AddPackage(String),
    RemovePackage(String),
    RemoveLibrary(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LanguageFileKind {
    Library(String),
    Template,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedLanguageFile {
    pub kind: LanguageFileKind,
    pub editor: String,
    pub path: PathBuf,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigField {
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
pub(crate) enum NonSecretConfigField {
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
pub(crate) struct ConfigData {
    pub year: String,
    pub editor: Option<String>,
    pub run_history_limit: String,
    pub session_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigOperationState {
    Idle,
    Loading,
    Saving,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct SecretString(String);

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
pub(crate) struct SecretCharacter(pub(crate) char);

impl std::fmt::Debug for SecretCharacter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SecretCharacter([REDACTED])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigDialog {
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
pub(crate) enum ConfigMutation {
    Set {
        field: NonSecretConfigField,
        value: Option<String>,
    },
    SetSession(SecretString),
    RemoveSession,
}

pub(crate) struct App {
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
    pub submission_dialog: Option<SubmissionDialog>,
    pub active_submission: Option<SubmissionRequest>,
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
    lazygit_preparing: bool,
    lazygit_opening: Option<bool>,
    lazygit_error: Option<String>,
    quit_after_config_save: bool,
    pub status: Option<String>,
    pub should_quit: bool,
}

#[derive(Debug)]
pub(crate) enum Action {
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
    OpenLazygit,
    RunPart(PuzzlePart),
    ToggleRunInput,
    Tick,
    CancelRunDialog,
    ScrollRunUp,
    ScrollRunDown,
    OpenSubmission,
    ToggleSubmissionChoice,
    SubmissionInput(char),
    SubmissionBackspace,
    SubmissionSubmit,
    SubmissionCancel,
    ScrollSubmissionUp,
    ScrollSubmissionDown,
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
    SubmissionFinished {
        request: SubmissionRequest,
        result: Result<AocSubmissionResult, String>,
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
    LazygitPrepared {
        language_active: bool,
        result: Result<PathBuf, String>,
    },
    ConfigLoaded {
        result: Result<ConfigData, String>,
    },
    ConfigSaved {
        result: Result<ConfigData, String>,
    },
    BackgroundSubmissionFailed {
        effect: BackgroundEffect,
        message: String,
    },
    ForegroundFinished(Result<(), String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BackgroundEffect {
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
    PrepareLazygit {
        language_active: bool,
    },
    RunSolver(RunRequest),
    SubmitAnswer(SubmissionRequest),
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
pub(crate) enum ForegroundEffect {
    Browser(PuzzleId),
    Exercise(PreparedExercise),
    LanguageFile(PreparedLanguageFile),
    Lazygit(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedExercise {
    pub editor: String,
    pub puzzle_description: PathBuf,
    pub example: PathBuf,
    pub solution: PathBuf,
    pub input: PathBuf,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
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
            submission_dialog: None,
            active_submission: None,
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
            lazygit_preparing: false,
            lazygit_opening: None,
            lazygit_error: None,
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
        if self.active_submission.is_some()
            && !matches!(
                action,
                Action::Tick
                    | Action::SubmissionFinished { .. }
                    | Action::BackgroundSubmissionFailed { .. }
                    | Action::CalendarFinished { .. }
                    | Action::CachedDescriptionFinished { .. }
                    | Action::DescriptionDownloaded { .. }
            )
        {
            return Vec::new();
        }
        if self.active_run.is_some()
            && !matches!(
                action,
                Action::Tick
                    | Action::RunFinished { .. }
                    | Action::BackgroundSubmissionFailed { .. }
                    | Action::CalendarFinished { .. }
                    | Action::CachedDescriptionFinished { .. }
                    | Action::DescriptionDownloaded { .. }
                    | Action::ExercisePrepared { .. }
                    | Action::LanguageDataFinished { .. }
                    | Action::LanguageMutationFinished { .. }
                    | Action::LanguageFilePrepared { .. }
                    | Action::ConfigLoaded { .. }
                    | Action::ConfigSaved { .. }
                    | Action::ForegroundFinished(_)
            )
        {
            if matches!(action, Action::Quit | Action::OpenLazygit) {
                self.status = Some(
                    if matches!(action, Action::OpenLazygit) {
                        "Wait for the solver run to finish before opening lazygit"
                    } else {
                        "Wait for the solver run to finish"
                    }
                    .to_owned(),
                );
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
            Action::OpenLazygit => {
                if self.lazygit_preparing {
                    self.status = Some("Workspace Git preparation is already running".to_owned());
                } else if self.exercise_preparing {
                    self.status = Some(
                        "Wait for exercise preparation to finish before opening lazygit".to_owned(),
                    );
                } else if self.language_busy() {
                    self.status = Some(
                        "Wait for the language operation to finish before opening lazygit"
                            .to_owned(),
                    );
                } else {
                    let language_active = self.active_tab == Tab::Language;
                    self.lazygit_preparing = true;
                    self.status = Some("Preparing workspace Git...".to_owned());
                    return vec![Effect::Background(BackgroundEffect::PrepareLazygit {
                        language_active,
                    })];
                }
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
                return vec![Effect::Foreground(ForegroundEffect::Browser(puzzle))];
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
            Action::Tick if self.active_run.is_some() || self.active_submission.is_some() => {
                self.run_spinner_frame = (self.run_spinner_frame + 1) % 4;
            }
            Action::CancelRunDialog => self.run_dialog = None,
            Action::ScrollRunUp => {
                if let Some(dialog) = &mut self.run_dialog {
                    dialog.scroll = dialog.scroll.saturating_sub(1);
                }
            }
            Action::ScrollRunDown => {
                if let Some(dialog) = &mut self.run_dialog {
                    dialog.scroll = dialog.scroll.saturating_add(1);
                }
            }
            Action::OpenSubmission => {
                if self.submission_dialog.is_some() || self.active_submission.is_some() {
                    return Vec::new();
                }
                if let Some(request) = self.run_submission_request() {
                    self.submission_dialog = Some(SubmissionDialog::Confirm {
                        request,
                        submit: false,
                    });
                } else if self.run_dialog.is_some() {
                    self.status = Some("This run result cannot be submitted".to_owned());
                } else if self.active_tab == Tab::Calendar {
                    let Some(puzzle) = self.selected_puzzle_or_status() else {
                        return Vec::new();
                    };
                    self.submission_dialog = Some(SubmissionDialog::Part {
                        puzzle,
                        part: PuzzlePart::One,
                    });
                }
            }
            Action::ToggleSubmissionChoice => match &mut self.submission_dialog {
                Some(SubmissionDialog::Part { part, .. }) => {
                    *part = match part {
                        PuzzlePart::One => PuzzlePart::Two,
                        PuzzlePart::Two => PuzzlePart::One,
                    };
                }
                Some(SubmissionDialog::Confirm { submit, .. }) => *submit = !*submit,
                _ => {}
            },
            Action::SubmissionInput(character) => {
                if let Some(SubmissionDialog::Answer { answer, error, .. }) =
                    &mut self.submission_dialog
                {
                    answer.push(character);
                    *error = None;
                }
            }
            Action::SubmissionBackspace => {
                if let Some(SubmissionDialog::Answer { answer, .. }) = &mut self.submission_dialog {
                    answer.pop();
                }
            }
            Action::SubmissionSubmit => return self.submit_submission_dialog(),
            Action::SubmissionCancel => self.submission_dialog = None,
            Action::ScrollSubmissionUp => {
                if let Some(SubmissionDialog::Outcome { scroll, .. }) = &mut self.submission_dialog
                {
                    *scroll = scroll.saturating_sub(1);
                }
            }
            Action::ScrollSubmissionDown => {
                if let Some(SubmissionDialog::Outcome { scroll, .. }) = &mut self.submission_dialog
                {
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
                        return vec![Effect::Foreground(ForegroundEffect::Exercise(prepared))];
                    }
                    Err(message) => self.status = Some(message),
                }
            }
            Action::RunFinished { request, result } => {
                self.active_run = None;
                self.status = None;
                self.run_dialog = Some(RunDialog {
                    request,
                    result,
                    scroll: 0,
                });
            }
            Action::SubmissionFinished { request, result } => {
                self.active_submission = None;
                let correct = matches!(result, Ok(AocSubmissionResult::Correct));
                self.submission_dialog = Some(SubmissionDialog::Outcome {
                    puzzle: request.puzzle,
                    part: request.part,
                    result,
                    scroll: 0,
                });
                if correct {
                    return self.submission_refreshes(request.puzzle, request.part);
                }
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
                if let Some(message) = self.lazygit_error.take() {
                    self.status = Some(message);
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
                        return vec![Effect::Foreground(ForegroundEffect::LanguageFile(prepared))];
                    }
                    Err(message) => self.show_language_error(message),
                }
            }
            Action::LazygitPrepared {
                language_active,
                result,
            } => {
                self.lazygit_preparing = false;
                match result {
                    Ok(path) => {
                        self.lazygit_opening = Some(language_active);
                        self.status = None;
                        return vec![Effect::Foreground(ForegroundEffect::Lazygit(path))];
                    }
                    Err(message) => self.status = Some(message),
                }
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
            Action::BackgroundSubmissionFailed { effect, message } => {
                self.background_submission_failed(effect, message);
            }
            Action::ForegroundFinished(result) => {
                if let Some(language_active) = self.lazygit_opening.take() {
                    self.lazygit_error = result.err();
                    if language_active {
                        return self.load_language_data();
                    }
                    self.status = self.lazygit_error.take();
                } else if let Some(kind) = self.language_file_opening.take() {
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

    fn run_submission_request(&self) -> Option<SubmissionRequest> {
        let dialog = self.run_dialog.as_ref()?;
        let Ok(report) = &dialog.result else {
            return None;
        };
        let part = report.parts.as_slice();
        if dialog.request.input != RunInput::Aoc
            || part.len() != 1
            || part[0].answer.trim().is_empty()
        {
            return None;
        }
        Some(SubmissionRequest::new(
            dialog.request.puzzle,
            part[0].part,
            part[0].answer.clone(),
        ))
    }

    fn background_submission_failed(&mut self, effect: BackgroundEffect, message: String) {
        match effect {
            BackgroundEffect::LoadCalendar { .. } => {
                self.calendar_loading = false;
                self.status = Some(message);
            }
            BackgroundEffect::LoadCachedDescription(puzzle) => {
                self.update(Action::CachedDescriptionFinished {
                    puzzle,
                    result: Err(message),
                });
            }
            BackgroundEffect::DownloadDescription(puzzle) => {
                self.update(Action::DescriptionDownloaded {
                    puzzle,
                    result: Err(message),
                });
            }
            BackgroundEffect::PrepareExercise { .. } => {
                self.exercise_preparing = false;
                self.status = Some(message);
            }
            BackgroundEffect::PrepareLazygit { .. } => {
                self.lazygit_preparing = false;
                self.status = Some(format!(
                    "Could not queue workspace Git preparation: {message}"
                ));
            }
            BackgroundEffect::RunSolver(request) => {
                self.active_run = None;
                self.status = None;
                self.run_dialog = Some(RunDialog {
                    request,
                    result: Err(RunFailure {
                        summary: "Could not queue solver run".to_owned(),
                        details: Some(message),
                    }),
                    scroll: 0,
                });
            }
            BackgroundEffect::SubmitAnswer(request) => {
                self.active_submission = None;
                self.submission_dialog = Some(SubmissionDialog::Outcome {
                    puzzle: request.puzzle,
                    part: request.part,
                    result: Err(message),
                    scroll: 0,
                });
            }
            BackgroundEffect::LoadLanguageData { .. }
            | BackgroundEffect::MutateLanguage { .. }
            | BackgroundEffect::PrepareLanguageFile { .. } => {
                self.language_file_opening = None;
                self.show_language_error(message);
            }
            BackgroundEffect::LoadConfig { .. } | BackgroundEffect::MutateConfig { .. } => {
                self.quit_after_config_save = false;
                self.show_config_error(message);
            }
        }
    }

    fn submit_submission_dialog(&mut self) -> Vec<Effect> {
        let Some(dialog) = self.submission_dialog.take() else {
            return Vec::new();
        };
        match dialog {
            SubmissionDialog::Part { puzzle, part } => {
                self.submission_dialog = Some(SubmissionDialog::Answer {
                    puzzle,
                    part,
                    answer: String::new(),
                    error: None,
                });
                Vec::new()
            }
            SubmissionDialog::Answer {
                puzzle,
                part,
                answer,
                ..
            } => {
                let answer = answer.trim().to_owned();
                if answer.is_empty() {
                    self.submission_dialog = Some(SubmissionDialog::Answer {
                        puzzle,
                        part,
                        answer,
                        error: Some("Answer cannot be empty".to_owned()),
                    });
                    return Vec::new();
                }
                self.start_submission(SubmissionRequest::new(puzzle, part, answer))
            }
            SubmissionDialog::Confirm { request, submit } if submit => {
                self.run_dialog = None;
                self.start_submission(request)
            }
            SubmissionDialog::Confirm { .. } | SubmissionDialog::Outcome { .. } => Vec::new(),
        }
    }

    fn start_submission(&mut self, request: SubmissionRequest) -> Vec<Effect> {
        if self.active_submission.is_some() {
            return Vec::new();
        }
        self.active_submission = Some(request.clone());
        vec![Effect::Background(BackgroundEffect::SubmitAnswer(request))]
    }

    fn submission_refreshes(&mut self, puzzle: PuzzleId, part: PuzzlePart) -> Vec<Effect> {
        if self.active_tab != Tab::Calendar {
            return Vec::new();
        }
        let mut effects = Vec::new();
        if puzzle.year == self.selected_year {
            self.calendar_loading = true;
            effects.push(Effect::Background(BackgroundEffect::LoadCalendar {
                year: puzzle.year,
                refresh: true,
            }));
        }
        if part == PuzzlePart::One && self.selected_puzzle == Some(puzzle) {
            self.description_downloads.insert(puzzle);
            effects.push(Effect::Background(BackgroundEffect::DownloadDescription(
                puzzle,
            )));
        }
        effects
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
        self.config_operation != ConfigOperationState::Idle
    }

    fn config_saving(&self) -> bool {
        self.config_operation == ConfigOperationState::Saving
    }

    fn load_config(&mut self) -> Vec<Effect> {
        self.config_operation = ConfigOperationState::Loading;
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
        self.config_operation = ConfigOperationState::Saving;
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
            || self.lazygit_preparing
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
    use aocsuite_parser::{AocSubmissionResult, Calendar, CalendarCell, CalendarRow, Rgb};
    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear};

    use super::{
        Action, App, BackgroundEffect, ConfigData, ConfigDialog, ConfigMutation,
        ConfigOperationState, DescriptionState, Effect, ForegroundEffect, LanguageData,
        LanguageDialog, LanguageFileKind, LanguageOperationState, NonSecretConfigField,
        PreparedExercise, RunDialog, RunInput, RunPartReport, RunReport, SubmissionDialog,
        SubmissionRequest,
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

    #[test]
    fn run_submission_confirmation_dispatches_the_retained_aoc_result() {
        let mut app = app();
        let retained = puzzle(3, 2025);
        let request = super::RunRequest {
            puzzle: retained,
            language: LanguageId::Python,
            part: PuzzlePart::Two,
            input: RunInput::Aoc,
        };
        app.run_dialog = Some(RunDialog {
            request,
            result: Ok(RunReport {
                compile_stdout: String::new(),
                compile_stderr: String::new(),
                solver_stdout: String::new(),
                solver_stderr: String::new(),
                parts: vec![RunPartReport {
                    part: PuzzlePart::Two,
                    answer: "retained-answer".to_owned(),
                    runtime_ms: 1,
                }],
                warning: None,
            }),
            scroll: 0,
        });
        app.update(Action::OpenSubmission);
        assert!(matches!(
            app.submission_dialog,
            Some(SubmissionDialog::Confirm {
                ref request,
                submit: false,
            }) if request.puzzle == retained
                && request.part == PuzzlePart::Two
                && request.answer() == "retained-answer"
        ));
        app.update(Action::ToggleSubmissionChoice);
        assert_eq!(
            app.update(Action::SubmissionSubmit),
            vec![Effect::Background(BackgroundEffect::SubmitAnswer(
                SubmissionRequest::new(retained, PuzzlePart::Two, "retained-answer".to_owned())
            ))]
        );
        assert!(app.active_submission.is_some());
    }

    #[test]
    fn submission_debug_redacts_manual_answers_through_dialog_actions_and_effects() {
        let sensitive = "sensitive-manual-answer";
        let puzzle = puzzle(10, 2026);
        let dialog = SubmissionDialog::Answer {
            puzzle,
            part: PuzzlePart::One,
            answer: sensitive.to_owned(),
            error: Some("validation context".to_owned()),
        };
        let request = SubmissionRequest {
            puzzle,
            part: PuzzlePart::One,
            answer: sensitive.to_owned(),
        };
        let action = Action::SubmissionFinished {
            request: request.clone(),
            result: Ok(AocSubmissionResult::Incorrect),
        };
        let effect = BackgroundEffect::SubmitAnswer(request.clone());
        let queue_failure = Action::BackgroundSubmissionFailed {
            effect: BackgroundEffect::SubmitAnswer(request),
            message: "worker stopped".to_owned(),
        };

        for debug in [
            format!("{dialog:?}"),
            format!("{action:?}"),
            format!("{effect:?}"),
            format!("{queue_failure:?}"),
        ] {
            assert!(!debug.contains(sensitive));
            assert!(debug.contains("[REDACTED]"));
        }
        assert!(format!("{dialog:?}").contains("validation context"));
    }

    #[test]
    fn active_submission_blocks_duplicates_and_correct_refreshes_visible_content() {
        let mut app = app();
        let selected = puzzle(10, 2026);
        app.selected_puzzle = Some(selected);
        let request = SubmissionRequest {
            puzzle: selected,
            part: PuzzlePart::One,
            answer: "answer".to_owned(),
        };
        app.active_submission = Some(request.clone());
        assert!(app.update(Action::OpenSubmission).is_empty());
        let effects = app.update(Action::SubmissionFinished {
            request,
            result: Ok(AocSubmissionResult::Correct),
        });
        assert_eq!(effects.len(), 2);
        assert!(effects.iter().any(|effect| matches!(
            effect,
            Effect::Background(BackgroundEffect::LoadCalendar { refresh: true, .. })
        )));
        assert!(effects.iter().any(|effect| matches!(
            effect,
            Effect::Background(BackgroundEffect::DownloadDescription(puzzle)) if *puzzle == selected
        )));
        assert!(matches!(
            app.submission_dialog,
            Some(SubmissionDialog::Outcome {
                result: Ok(AocSubmissionResult::Correct),
                ..
            })
        ));
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
    fn stale_download_updates_no_visible_state_and_releases_its_guard() {
        let mut app = selected_app();
        let stale = app.selected_puzzle().unwrap();
        app.update(Action::DownloadDescription);
        app.update(Action::NextCalendarPuzzle);
        let selected = app.selected_puzzle().unwrap();

        app.update(Action::CachedDescriptionFinished {
            puzzle: stale,
            result: Ok(Some("stale preview".to_owned())),
        });
        assert_eq!(app.description, DescriptionState::CheckingCache(selected));
        assert_eq!(app.selected_puzzle(), Some(selected));

        app.update(Action::DescriptionDownloaded {
            puzzle: stale,
            result: Ok("updated stale puzzle".to_owned()),
        });

        assert_eq!(app.description, DescriptionState::CheckingCache(selected));
        assert!(!app.description_downloading(stale));
        assert_eq!(app.selected_puzzle(), Some(selected));
        assert!(app.status.is_none());
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
        assert!(app.update(Action::RunPart(PuzzlePart::Two)).is_empty());
    }
    #[test]
    fn destructive_confirmations_cancel_by_default() {
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

        app.update(Action::ConfigSaved {
            result: Ok(ConfigData {
                year: "2026".to_owned(),
                editor: Some("vim".to_owned()),
                run_history_limit: "10".to_owned(),
                session_configured: false,
            }),
        });
        assert!(app.should_quit);
    }
}
