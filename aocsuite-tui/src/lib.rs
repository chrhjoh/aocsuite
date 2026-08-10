mod app;
mod effects;
mod terminal;
mod ui;

use std::time::Duration;

use aocsuite_client::AocClientError;
use aocsuite_config::{AocConfigError, ConfigKey, Configuration};
use aocsuite_lang::AocLanguageError;
use aocsuite_launcher::AocLauncherError;
use aocsuite_parser::ParserError;
use aocsuite_storage::{
    get_aocsuite_dir, ContentError, LayoutError, RuntimeLayout, Workspace, WorkspaceError,
};
use aocsuite_utils::{
    default_puzzle_date, LanguageId, PuzzleId, ReleaseError, SystemCommandExecutor,
};
use app::{
    Action, App, ConfigDialog, Effect, LanguageDialog, SecretCharacter, SubmissionDialog, Tab,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use effects::{run_foreground_effect, EffectRunner};
use terminal::TerminalSession;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TuiError {
    #[error(transparent)]
    Client(#[from] AocClientError),
    #[error(transparent)]
    Config(#[from] AocConfigError),
    #[error(transparent)]
    Content(#[from] ContentError),
    #[error(transparent)]
    Language(#[from] AocLanguageError),
    #[error(transparent)]
    Launcher(#[from] AocLauncherError),
    #[error(transparent)]
    Layout(#[from] LayoutError),
    #[error(transparent)]
    Parser(#[from] ParserError),
    #[error(transparent)]
    Release(#[from] ReleaseError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("the background effect runner stopped unexpectedly")]
    EffectRunnerStopped,
}

pub type TuiResult<T> = Result<T, TuiError>;

pub fn run() -> TuiResult<()> {
    let root = get_aocsuite_dir()?;
    let layout = RuntimeLayout::new(root)?;
    layout.bootstrap()?;
    let workspace = Workspace::new(layout.workspace_dir());
    workspace.ensure()?;
    let config = Configuration::load(layout.config_dir())?;
    let configured_year = match config.get(ConfigKey::Year) {
        Ok(year) => Some(year),
        Err(AocConfigError::NotFound { .. }) => None,
        Err(error) => return Err(error.into()),
    };
    let language = config.get::<LanguageId>(ConfigKey::Language)?;
    let (latest_day, latest_year) = default_puzzle_date();
    let mut app = App::new(
        configured_year,
        PuzzleId::new(latest_day, latest_year),
        language,
    );
    let effects = EffectRunner::new(layout.clone());
    for effect in app.initial_effects() {
        match effect {
            Effect::Background(effect) => effects.submit(effect).map_err(|(_, error)| error)?,
            Effect::Foreground(_) => unreachable!("initial effects are background work"),
        }
    }

    let mut terminal = TerminalSession::enter()?;
    let executor = SystemCommandExecutor;
    while !app.should_quit {
        while let Some(action) = effects.try_receive() {
            let requested = app.update(action);
            dispatch_effects(&mut app, &effects, &mut terminal, &executor, requested)?;
        }

        terminal
            .terminal_mut()
            .draw(|frame| ui::render(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = action_for_key(&app, key) {
                        let requested = app.update(action);
                        dispatch_effects(&mut app, &effects, &mut terminal, &executor, requested)?;
                    }
                }
            }
        }
        app.update(Action::Tick);
    }
    Ok(())
}

fn dispatch_effects(
    app: &mut App,
    runner: &EffectRunner,
    terminal: &mut TerminalSession,
    executor: &SystemCommandExecutor,
    effects: Vec<Effect>,
) -> TuiResult<()> {
    for effect in effects {
        match effect {
            Effect::Background(effect) => {
                if let Err((effect, error)) = runner.submit(effect) {
                    app.update(Action::BackgroundSubmissionFailed {
                        effect: *effect,
                        message: error.to_string(),
                    });
                }
            }
            Effect::Foreground(effect) => {
                terminal.suspend()?;
                let result =
                    run_foreground_effect(effect, executor).map_err(|error| error.to_string());
                terminal.resume()?;
                let follow_up = app.update(Action::ForegroundFinished(result));
                dispatch_effects(app, runner, terminal, executor, follow_up)?;
            }
        }
    }
    Ok(())
}

fn action_for_key(app: &App, key: KeyEvent) -> Option<Action> {
    if app.active_run.is_some() {
        return match key.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('g') => Some(Action::OpenLazygit),
            _ => None,
        };
    }
    if app.active_submission.is_some() {
        return None;
    }
    if let Some(dialog) = &app.submission_dialog {
        return match dialog {
            SubmissionDialog::Part { part, .. } => match key.code {
                KeyCode::Esc => Some(Action::SubmissionCancel),
                KeyCode::Enter => Some(Action::SubmissionSubmit),
                KeyCode::Left if *part == aocsuite_utils::PuzzlePart::Two => {
                    Some(Action::ToggleSubmissionChoice)
                }
                KeyCode::Right if *part == aocsuite_utils::PuzzlePart::One => {
                    Some(Action::ToggleSubmissionChoice)
                }
                KeyCode::Tab | KeyCode::BackTab => Some(Action::ToggleSubmissionChoice),
                _ => None,
            },
            SubmissionDialog::Answer { .. } => match key.code {
                KeyCode::Esc => Some(Action::SubmissionCancel),
                KeyCode::Enter => Some(Action::SubmissionSubmit),
                KeyCode::Backspace => Some(Action::SubmissionBackspace),
                KeyCode::Char(character) => Some(Action::SubmissionInput(character)),
                _ => None,
            },
            SubmissionDialog::Confirm { submit, .. } => match key.code {
                KeyCode::Esc => Some(Action::SubmissionCancel),
                KeyCode::Enter => Some(Action::SubmissionSubmit),
                KeyCode::Left if *submit => Some(Action::ToggleSubmissionChoice),
                KeyCode::Right if !*submit => Some(Action::ToggleSubmissionChoice),
                KeyCode::Tab | KeyCode::BackTab => Some(Action::ToggleSubmissionChoice),
                _ => None,
            },
            SubmissionDialog::Outcome { .. } => match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Action::SubmissionCancel),
                KeyCode::Up | KeyCode::PageUp => Some(Action::ScrollSubmissionUp),
                KeyCode::Down | KeyCode::PageDown => Some(Action::ScrollSubmissionDown),
                _ => None,
            },
        };
    }
    if app.run_dialog.is_some() {
        return match key.code {
            KeyCode::Char('s') => Some(Action::OpenSubmission),
            KeyCode::Esc | KeyCode::Enter => Some(Action::CancelRunDialog),
            KeyCode::Up | KeyCode::PageUp => Some(Action::ScrollRunUp),
            KeyCode::Down | KeyCode::PageDown => Some(Action::ScrollRunDown),
            _ => None,
        };
    }
    if let Some(dialog) = &app.config_dialog {
        return match dialog {
            ConfigDialog::Text { .. } => match key.code {
                KeyCode::Esc => Some(Action::ConfigCancel),
                KeyCode::Enter => Some(Action::ConfigSubmit),
                KeyCode::Backspace => Some(Action::ConfigBackspace),
                KeyCode::Char(character) => Some(Action::ConfigInput(character)),
                _ => None,
            },
            ConfigDialog::Session { .. } => match key.code {
                KeyCode::Esc => Some(Action::ConfigCancel),
                KeyCode::Enter => Some(Action::ConfigSubmit),
                KeyCode::Backspace => Some(Action::ConfigBackspace),
                KeyCode::Char(character) => {
                    Some(Action::ConfigSecretInput(SecretCharacter(character)))
                }
                _ => None,
            },
            ConfigDialog::ConfirmRemoveSession { confirmed } => match key.code {
                KeyCode::Esc => Some(Action::ConfigCancel),
                KeyCode::Enter => Some(Action::ConfigSubmit),
                KeyCode::Left if *confirmed => Some(Action::ConfigToggleConfirmation),
                KeyCode::Right if !*confirmed => Some(Action::ConfigToggleConfirmation),
                KeyCode::Tab | KeyCode::BackTab => Some(Action::ConfigToggleConfirmation),
                _ => None,
            },
            ConfigDialog::Message { .. } => match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Action::ConfigCancel),
                KeyCode::Up | KeyCode::PageUp => Some(Action::ConfigScrollMessageUp),
                KeyCode::Down | KeyCode::PageDown => Some(Action::ConfigScrollMessageDown),
                _ => None,
            },
        };
    }

    if let Some(dialog) = &app.language_dialog {
        return match dialog {
            LanguageDialog::Text { .. } => match key.code {
                KeyCode::Esc => Some(Action::DialogCancel),
                KeyCode::Enter => Some(Action::DialogSubmit),
                KeyCode::Backspace => Some(Action::DialogBackspace),
                KeyCode::Char(character) => Some(Action::DialogInput(character)),
                _ => None,
            },
            LanguageDialog::Confirm { confirmed, .. } => match key.code {
                KeyCode::Esc => Some(Action::DialogCancel),
                KeyCode::Enter => Some(Action::DialogSubmit),
                KeyCode::Left if *confirmed => Some(Action::DialogToggleConfirmation),
                KeyCode::Right if !*confirmed => Some(Action::DialogToggleConfirmation),
                KeyCode::Tab | KeyCode::BackTab => Some(Action::DialogToggleConfirmation),
                _ => None,
            },
            LanguageDialog::Message(_) => match key.code {
                KeyCode::Esc | KeyCode::Enter => Some(Action::DialogCancel),
                _ => None,
            },
        };
    }

    if app.help_open {
        return match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) => Some(Action::Quit),
            (KeyCode::Tab, KeyModifiers::SHIFT) | (KeyCode::BackTab, _) => {
                Some(Action::PreviousTab)
            }
            (KeyCode::Tab, _) => Some(Action::NextTab),
            (KeyCode::Esc | KeyCode::Enter | KeyCode::Char('?'), _) => Some(Action::CloseHelp),
            (KeyCode::Up | KeyCode::PageUp, _) => Some(Action::ScrollHelpUp),
            (KeyCode::Down | KeyCode::PageDown, _) => Some(Action::ScrollHelpDown),
            _ => None,
        };
    }

    match (key.code, key.modifiers) {
        (KeyCode::Char('?'), _) => return Some(Action::OpenHelp),
        (KeyCode::Char('q'), _) => return Some(Action::Quit),
        (KeyCode::Char('g'), _) => return Some(Action::OpenLazygit),
        (KeyCode::Tab, KeyModifiers::SHIFT) | (KeyCode::BackTab, _) => {
            return Some(Action::PreviousTab);
        }
        (KeyCode::Tab, _) => return Some(Action::NextTab),
        _ => {}
    }

    if app.active_tab == Tab::Language {
        return match key.code {
            KeyCode::Char('s') => Some(Action::SwitchLanguage),
            KeyCode::Char('r') => Some(Action::RefreshLanguage),
            KeyCode::Left => Some(Action::PreviousLanguagePane),
            KeyCode::Right => Some(Action::NextLanguagePane),
            KeyCode::Up => Some(Action::PreviousLanguageItem),
            KeyCode::Down => Some(Action::NextLanguageItem),
            KeyCode::Char('a') => Some(Action::AddPackage),
            KeyCode::Char('x') => Some(Action::RemoveLanguageItem),
            KeyCode::Char('n') => Some(Action::NewLibrary),
            KeyCode::Enter => Some(Action::OpenLanguageItem),
            KeyCode::Char('T') => Some(Action::ResetTemplate),
            KeyCode::Char('t') => Some(Action::OpenTemplate),
            _ => None,
        };
    }

    if app.active_tab == Tab::Config {
        return match key.code {
            KeyCode::Char('r') => Some(Action::RefreshConfig),
            KeyCode::Up => Some(Action::PreviousConfigField),
            KeyCode::Down => Some(Action::NextConfigField),
            KeyCode::Enter => Some(Action::EditConfigField),
            KeyCode::Char('x') => Some(Action::RemoveConfigValue),
            _ => None,
        };
    }

    match (key.code, key.modifiers) {
        (KeyCode::Up, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarUp),
        (KeyCode::Down, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarDown),
        (KeyCode::Left, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarLeft),
        (KeyCode::Right, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarRight),
        (KeyCode::Left, _) => Some(Action::PreviousYear),
        (KeyCode::Right, _) => Some(Action::NextYear),
        (KeyCode::Up, _) => Some(Action::PreviousCalendarPuzzle),
        (KeyCode::Down, _) => Some(Action::NextCalendarPuzzle),
        (KeyCode::Char('d'), _) => Some(Action::DownloadDescription),
        (KeyCode::Char('s'), _) => Some(Action::OpenSubmission),
        (KeyCode::Char('1'), _) => Some(Action::RunPart(aocsuite_utils::PuzzlePart::One)),
        (KeyCode::Char('2'), _) => Some(Action::RunPart(aocsuite_utils::PuzzlePart::Two)),
        (KeyCode::Char('i'), _) => Some(Action::ToggleRunInput),
        (KeyCode::Char('u'), _) => Some(Action::RefreshCalendar),
        (KeyCode::Char('b'), _) => Some(Action::OpenBrowser),
        (KeyCode::Enter, _) => Some(Action::OpenExercise),
        (KeyCode::PageUp, _) => Some(Action::ScrollDescriptionUp),
        (KeyCode::PageDown, _) => Some(Action::ScrollDescriptionDown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzleYear};

    use super::{action_for_key, Action, App};
    use crate::app::LanguageData;

    fn app() -> App {
        App::new(
            None,
            PuzzleId::new(PuzzleDay::new(1).unwrap(), PuzzleYear::new(2026).unwrap()),
            LanguageId::Rust,
        )
    }

    #[test]
    fn modal_input_takes_priority_over_global_shortcuts() {
        let mut app = app();
        app.update(Action::NextTab);
        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Ok(LanguageData {
                packages: vec![],
                libraries: vec![],
            }),
        });
        app.update(Action::AddPackage);

        assert!(matches!(
            action_for_key(&app, KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            Some(Action::DialogInput('q'))
        ));
        assert!(matches!(
            action_for_key(&app, KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE)),
            Some(Action::DialogInput('g'))
        ));
        assert!(matches!(
            action_for_key(&app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Action::DialogCancel)
        ));
    }
}
