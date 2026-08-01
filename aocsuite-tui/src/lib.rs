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
pub use app::{Action, App, BackgroundEffect, Effect, ForegroundEffect, PreparedExercise, Tab};
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
    effects.submit(match app.initial_effect() {
        Effect::Background(effect) => effect,
        Effect::Foreground(_) => unreachable!("initial effect is background work"),
    })?;

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
                    if let Some(action) = action_for_key(key) {
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
                let was_exercise = matches!(&effect, BackgroundEffect::PrepareExercise(_));
                if let Err(error) = runner.submit(effect) {
                    if was_exercise {
                        app.exercise_preparing = false;
                    }
                    app.update(Action::EffectFailed(error.to_string()));
                }
            }
            Effect::Foreground(effect) => {
                terminal.suspend()?;
                let result =
                    run_foreground_effect(effect, executor).map_err(|error| error.to_string());
                terminal.resume()?;
                app.update(Action::ForegroundFinished(result));
            }
        }
    }
    Ok(())
}

fn action_for_key(key: KeyEvent) -> Option<Action> {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) => Some(Action::Quit),
        (KeyCode::Tab, KeyModifiers::SHIFT) | (KeyCode::BackTab, _) => Some(Action::PreviousTab),
        (KeyCode::Tab, _) => Some(Action::NextTab),
        (KeyCode::Up, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarUp),
        (KeyCode::Down, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarDown),
        (KeyCode::Left, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarLeft),
        (KeyCode::Right, KeyModifiers::CONTROL) => Some(Action::ScrollCalendarRight),
        (KeyCode::Left, _) => Some(Action::PreviousYear),
        (KeyCode::Right, _) => Some(Action::NextYear),
        (KeyCode::Up, _) => Some(Action::PreviousDay),
        (KeyCode::Down, _) => Some(Action::NextDay),
        (KeyCode::Char('d'), _) => Some(Action::LoadDescription),
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

    use super::{action_for_key, Action};

    #[test]
    fn key_mapping_keeps_event_handling_separate_from_state_updates() {
        assert!(matches!(
            action_for_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)),
            Some(Action::LoadDescription)
        ));
        assert!(matches!(
            action_for_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(Action::PreviousTab)
        ));
    }
}
