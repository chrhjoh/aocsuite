use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use aocsuite_client::{AocClient, AocClientOptions, AocPage};
use aocsuite_config::{AocConfigError, ConfigKey, Configuration};
use aocsuite_lang::{Language, SolverFile};
use aocsuite_launcher::{Launcher, OpenPuzzleRequest};
use aocsuite_parser::parse_calendar;
use aocsuite_storage::{ContentStore, RuntimeLayout, Workspace};
use aocsuite_utils::{
    valid_puzzle_release, CommandExecutor, LanguageId, PuzzleId, SystemCommandExecutor,
};

use crate::{
    app::{Action, BackgroundEffect, ForegroundEffect, PreparedExercise},
    TuiError,
};

pub struct EffectRunner {
    sender: Option<mpsc::Sender<BackgroundEffect>>,
    receiver: mpsc::Receiver<Action>,
    worker: Option<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl EffectRunner {
    pub fn new(layout: RuntimeLayout) -> Self {
        let (effect_sender, effect_receiver) = mpsc::channel();
        let (action_sender, action_receiver) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);
        let worker = thread::spawn(move || {
            worker_loop(effect_receiver, action_sender, worker_shutdown, |effect| {
                run_background_effect(&layout, effect)
            });
        });
        Self {
            sender: Some(effect_sender),
            receiver: action_receiver,
            worker: Some(worker),
            shutdown,
        }
    }

    pub fn submit(&self, effect: BackgroundEffect) -> Result<(), TuiError> {
        self.sender
            .as_ref()
            .expect("effect sender exists until drop")
            .send(effect)
            .map_err(|_| TuiError::EffectRunnerStopped)
    }

    pub fn try_receive(&self) -> Option<Action> {
        self.receiver.try_recv().ok()
    }
}

fn worker_loop(
    effect_receiver: mpsc::Receiver<BackgroundEffect>,
    action_sender: mpsc::Sender<Action>,
    shutdown: Arc<AtomicBool>,
    mut run: impl FnMut(BackgroundEffect) -> Action,
) {
    while let Ok(effect) = effect_receiver.recv() {
        if shutdown.load(Ordering::Acquire) {
            break;
        }
        if action_sender.send(run(effect)).is_err() {
            break;
        }
    }
}

impl Drop for EffectRunner {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run_background_effect(layout: &RuntimeLayout, effect: BackgroundEffect) -> Action {
    match effect {
        BackgroundEffect::LoadCalendar { year, refresh } => {
            let result = with_content_store(layout, |content| {
                let html = if refresh {
                    content.refresh_calendar(year)?
                } else {
                    content.load_calendar(year)?
                };
                Ok(parse_calendar(&html)?)
            })
            .map_err(|error| format!("Could not load calendar {year}: {error}"));
            Action::CalendarFinished {
                year,
                refresh,
                result,
            }
        }
        BackgroundEffect::LoadDescription(puzzle) => {
            let result =
                with_content_store(layout, |content| Ok(content.load_puzzle_markdown(puzzle)?))
                    .map_err(|error| format!("Could not load {puzzle}: {error}"));
            Action::DescriptionFinished { puzzle, result }
        }
        BackgroundEffect::PrepareExercise(puzzle) => {
            let executor = SystemCommandExecutor;
            let result = prepare_exercise(layout, puzzle, &executor)
                .map_err(|error| format!("Could not prepare {puzzle}: {error}"));
            Action::ExercisePrepared { puzzle, result }
        }
    }
}

fn with_content_store<T>(
    layout: &RuntimeLayout,
    operation: impl FnOnce(&ContentStore<'_>) -> Result<T, TuiError>,
) -> Result<T, TuiError> {
    let config = Configuration::load(layout.config_dir())?;
    let session = load_optional_session(&config)?;
    let client = AocClient::new(session.as_deref(), AocClientOptions::default())?;
    let content = ContentStore::open(layout.cache_dir(), &client)?;
    operation(&content)
}

fn load_optional_session(config: &Configuration) -> Result<Option<String>, AocConfigError> {
    match config.session() {
        Ok(session) => Ok(Some(session)),
        Err(AocConfigError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn run_foreground_effect(
    effect: ForegroundEffect,
    executor: &dyn CommandExecutor,
) -> Result<(), TuiError> {
    let launcher = Launcher::new(executor);
    match effect {
        ForegroundEffect::OpenBrowser(puzzle) => {
            valid_puzzle_release(puzzle.day, puzzle.year)?;
            launcher.open_browser(&AocPage::Puzzle(puzzle).to_string())?;
        }
        ForegroundEffect::OpenExercise(prepared) => {
            launcher.open_puzzle(
                prepared.editor,
                OpenPuzzleRequest {
                    puzzle: prepared.puzzle_description,
                    example: prepared.example,
                    solution: prepared.solution,
                    input: prepared.input,
                    working_directory: prepared.working_directory,
                },
            )?;
        }
    }
    Ok(())
}

fn prepare_exercise(
    layout: &RuntimeLayout,
    puzzle: PuzzleId,
    executor: &dyn CommandExecutor,
) -> Result<PreparedExercise, TuiError> {
    valid_puzzle_release(puzzle.day, puzzle.year)?;
    let config = Configuration::load(layout.config_dir())?;
    let session = load_optional_session(&config)?;
    let client = AocClient::new(session.as_deref(), AocClientOptions::default())?;
    let content = ContentStore::open(layout.cache_dir(), &client)?;
    let workspace = Workspace::new(layout.workspace_dir());
    let language_id = config.get::<LanguageId>(ConfigKey::Language)?;
    let language = Language::new(language_id, &workspace, executor);
    let editor = config.get::<String>(ConfigKey::Editor)?;
    Ok(PreparedExercise {
        puzzle,
        editor,
        puzzle_description: content.ensure_puzzle_markdown(puzzle)?,
        example: workspace.ensure_example(puzzle)?,
        solution: language.ensure_solver_file(&SolverFile::ActiveSolution(puzzle))?,
        input: content.ensure_input(puzzle)?,
        working_directory: language.project_dir().to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    };

    use aocsuite_utils::PuzzleYear;

    use super::worker_loop;
    use crate::app::{Action, BackgroundEffect};

    #[test]
    fn shutdown_abandons_effects_queued_after_active_work() {
        let (effect_sender, effect_receiver) = mpsc::channel();
        let (action_sender, _action_receiver) = mpsc::channel();
        let (started_sender, started_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = Arc::clone(&shutdown);
        let executions = Arc::new(AtomicUsize::new(0));
        let worker_executions = Arc::clone(&executions);
        let worker = std::thread::spawn(move || {
            worker_loop(effect_receiver, action_sender, worker_shutdown, move |_| {
                worker_executions.fetch_add(1, Ordering::Relaxed);
                started_sender.send(()).unwrap();
                release_receiver.recv().unwrap();
                Action::EffectFailed("test effect complete".to_owned())
            });
        });
        let effect = BackgroundEffect::LoadCalendar {
            year: PuzzleYear::new(2024).unwrap(),
            refresh: false,
        };
        effect_sender.send(effect.clone()).unwrap();
        effect_sender.send(effect).unwrap();
        started_receiver.recv().unwrap();

        shutdown.store(true, Ordering::Release);
        release_sender.send(()).unwrap();
        worker.join().unwrap();

        assert_eq!(executions.load(Ordering::Relaxed), 1);
    }
}
