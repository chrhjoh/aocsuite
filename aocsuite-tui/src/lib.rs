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
pub use app::{
    Action, App, BackgroundEffect, Effect, ForegroundEffect, LanguageConfirmation, LanguageData,
    LanguageDialog, LanguageFileKind, LanguageFocus, LanguageMutation, LanguageOperationState,
    LanguageTextInput, PreparedExercise, PreparedLanguageFile, Tab,
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
            Effect::Background(effect) => effects.submit(effect)?,
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
                let was_exercise = matches!(&effect, BackgroundEffect::PrepareExercise { .. });
                let was_language = matches!(
                    &effect,
                    BackgroundEffect::LoadLanguageData { .. }
                        | BackgroundEffect::MutateLanguage { .. }
                        | BackgroundEffect::PrepareLanguageFile { .. }
                );
                let cached_puzzle = match &effect {
                    BackgroundEffect::LoadCachedDescription(puzzle) => Some(*puzzle),
                    _ => None,
                };
                let downloaded_puzzle = match &effect {
                    BackgroundEffect::DownloadDescription(puzzle) => Some(*puzzle),
                    _ => None,
                };
                if let Err(error) = runner.submit(effect) {
                    let message = error.to_string();
                    if was_exercise {
                        app.exercise_preparing = false;
                    }
                    if was_language {
                        app.update(Action::LanguageEffectFailed(message));
                    } else if let Some(puzzle) = cached_puzzle {
                        app.update(Action::CachedDescriptionFinished {
                            puzzle,
                            result: Err(message),
                        });
                    } else if let Some(puzzle) = downloaded_puzzle {
                        app.update(Action::DescriptionDownloaded {
                            puzzle,
                            result: Err(message),
                        });
                    } else {
                        app.update(Action::EffectFailed(message));
                    }
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

    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => return Some(Action::Quit),
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
            KeyCode::Enter | KeyCode::Char('o') => Some(Action::OpenLanguageItem),
            KeyCode::Char('T') => Some(Action::ResetTemplate),
            KeyCode::Char('t') => Some(Action::OpenTemplate),
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
        (KeyCode::Char('r'), _) => Some(Action::RefreshCalendar),
        (KeyCode::Char('b'), _) => Some(Action::OpenBrowser),
        (KeyCode::Char('o'), _) | (KeyCode::Enter, _) => Some(Action::OpenExercise),
        (KeyCode::PageUp, _) => Some(Action::ScrollDescriptionUp),
        (KeyCode::PageDown, _) => Some(Action::ScrollDescriptionDown),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzleYear};

    use super::{action_for_key, Action, App, LanguageData, LanguageDialog};

    fn app() -> App {
        App::new(
            None,
            PuzzleId::new(PuzzleDay::new(1).unwrap(), PuzzleYear::new(2026).unwrap()),
            LanguageId::Rust,
        )
    }

    #[test]
    fn key_mapping_keeps_event_handling_separate_from_state_updates() {
        assert!(matches!(
            action_for_key(
                &app(),
                KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)
            ),
            Some(Action::DownloadDescription)
        ));
        assert!(matches!(
            action_for_key(&app(), KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(Action::PreviousTab)
        ));
        assert!(matches!(
            action_for_key(&app(), KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Some(Action::NextCalendarPuzzle)
        ));
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
            action_for_key(&app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Action::DialogCancel)
        ));
    }

    #[test]
    fn confirmation_arrows_preserve_cancel_as_the_left_choice() {
        let mut app = app();
        app.update(Action::NextTab);
        app.update(Action::LanguageDataFinished {
            language: LanguageId::Rust,
            result: Ok(LanguageData {
                packages: vec!["anyhow".to_owned()],
                libraries: vec![],
            }),
        });
        app.update(Action::RemoveLanguageItem);

        assert!(action_for_key(&app, KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)).is_none());
        let action =
            action_for_key(&app, KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)).unwrap();
        app.update(action);
        assert!(matches!(
            app.language_dialog,
            Some(LanguageDialog::Confirm {
                confirmed: true,
                ..
            })
        ));
    }
}
