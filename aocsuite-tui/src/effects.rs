use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use aocsuite_client::{AocClient, AocClientOptions, AocPage};
use aocsuite_config::{AocConfigError, ConfigKey, Configuration};
use aocsuite_lang::{ConfirmedLibraryRemoval, ConfirmedTemplateReset, Language, SolverFile};
use aocsuite_launcher::{Launcher, OpenPuzzleRequest};
use aocsuite_parser::parse_calendar;
use aocsuite_storage::{ContentStore, RuntimeLayout, Workspace};
use aocsuite_utils::{
    valid_puzzle_release, CommandExecutor, LanguageId, PuzzleId, SystemCommandExecutor,
};

use crate::{
    app::{
        Action, BackgroundEffect, ForegroundEffect, LanguageData, LanguageFileKind,
        LanguageMutation, PreparedExercise, PreparedLanguageFile,
    },
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
            let executor = SystemCommandExecutor;
            worker_loop(effect_receiver, action_sender, worker_shutdown, |effect| {
                run_background_effect(&layout, effect, &executor)
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

fn run_background_effect(
    layout: &RuntimeLayout,
    effect: BackgroundEffect,
    executor: &dyn CommandExecutor,
) -> Action {
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
        BackgroundEffect::LoadCachedDescription(puzzle) => {
            let result = with_content_store(layout, |content| {
                Ok(content.load_cached_puzzle_markdown(puzzle)?)
            })
            .map_err(|error| format!("Could not read cached {puzzle}: {error}"));
            Action::CachedDescriptionFinished { puzzle, result }
        }
        BackgroundEffect::DownloadDescription(puzzle) => {
            let result = with_content_store(layout, |content| {
                Ok(content.download_puzzle_markdown(puzzle)?)
            })
            .map_err(|error| format!("Could not download {puzzle}: {error}"));
            Action::DescriptionDownloaded { puzzle, result }
        }
        BackgroundEffect::PrepareExercise { puzzle, language } => {
            let result = prepare_exercise(layout, puzzle, language, executor)
                .map_err(|error| format!("Could not prepare {puzzle}: {error}"));
            Action::ExercisePrepared {
                puzzle,
                language,
                result,
            }
        }
        BackgroundEffect::LoadLanguageData { language } => {
            let result = load_language_data(layout, language, executor)
                .map_err(|error| format!("Could not load {language} language data: {error}"));
            Action::LanguageDataFinished { language, result }
        }
        BackgroundEffect::MutateLanguage { language, mutation } => {
            let result = mutate_language(layout, language, &mutation, executor).map_err(|error| {
                format!("Could not finish {}: {error}", mutation_action(&mutation))
            });
            Action::LanguageMutationFinished { language, result }
        }
        BackgroundEffect::PrepareLanguageFile {
            language,
            kind,
            reset,
        } => {
            let context = match &kind {
                LanguageFileKind::Library(name) => format!("library {name}"),
                LanguageFileKind::Template => "the template".to_owned(),
            };
            let result = prepare_language_file(layout, language, kind, reset, executor)
                .map_err(|error| format!("Could not prepare {context} editor: {error}"));
            Action::LanguageFilePrepared { language, result }
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
        ForegroundEffect::OpenLanguageFile(prepared) => {
            launcher.open_file(prepared.editor, &prepared.path, &prepared.working_directory)?;
        }
    }
    Ok(())
}

fn prepare_exercise(
    layout: &RuntimeLayout,
    puzzle: PuzzleId,
    language_id: LanguageId,
    executor: &dyn CommandExecutor,
) -> Result<PreparedExercise, TuiError> {
    valid_puzzle_release(puzzle.day, puzzle.year)?;
    let config = Configuration::load(layout.config_dir())?;
    let session = load_optional_session(&config)?;
    let client = AocClient::new(session.as_deref(), AocClientOptions::default())?;
    let content = ContentStore::open(layout.cache_dir(), &client)?;
    let workspace = Workspace::new(layout.workspace_dir());
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

fn load_language_data(
    layout: &RuntimeLayout,
    language_id: LanguageId,
    executor: &dyn CommandExecutor,
) -> Result<LanguageData, TuiError> {
    let workspace = Workspace::new(layout.workspace_dir());
    let language = Language::new(language_id, &workspace, executor);
    let mut packages = language.list_packages()?;
    let mut libraries = language.list_lib_files()?;
    packages.sort_unstable_by_key(|package| package.to_ascii_lowercase());
    libraries.sort_unstable_by_key(|library| library.to_ascii_lowercase());
    Ok(LanguageData {
        packages,
        libraries,
    })
}

fn mutate_language(
    layout: &RuntimeLayout,
    language_id: LanguageId,
    mutation: &LanguageMutation,
    executor: &dyn CommandExecutor,
) -> Result<LanguageData, TuiError> {
    let workspace = Workspace::new(layout.workspace_dir());
    let language = Language::new(language_id, &workspace, executor);
    match mutation {
        LanguageMutation::AddPackage(package) => language.add_package(package)?,
        LanguageMutation::RemovePackage(package) => language.remove_package(package)?,
        LanguageMutation::RemoveLibrary(library) => {
            language.remove_lib_file(library, ConfirmedLibraryRemoval::Confirmed)?;
        }
    }
    load_language_data(layout, language_id, executor)
}

fn mutation_action(mutation: &LanguageMutation) -> String {
    match mutation {
        LanguageMutation::AddPackage(package) => format!("adding package {package}"),
        LanguageMutation::RemovePackage(package) => format!("removing package {package}"),
        LanguageMutation::RemoveLibrary(library) => format!("removing library {library}"),
    }
}

fn prepare_language_file(
    layout: &RuntimeLayout,
    language_id: LanguageId,
    kind: LanguageFileKind,
    reset: bool,
    executor: &dyn CommandExecutor,
) -> Result<PreparedLanguageFile, TuiError> {
    let config = Configuration::load(layout.config_dir())?;
    let editor = config.get::<String>(ConfigKey::Editor)?;
    let workspace = Workspace::new(layout.workspace_dir());
    let language = Language::new(language_id, &workspace, executor);
    let path = match &kind {
        LanguageFileKind::Library(name) => language.ensure_lib_path(name)?,
        LanguageFileKind::Template if reset => {
            language.reset_template(ConfirmedTemplateReset::Confirmed)?
        }
        LanguageFileKind::Template => language.ensure_solver_file(&SolverFile::SolutionTemplate)?,
    };
    Ok(PreparedLanguageFile {
        language: language_id,
        kind,
        editor,
        path,
        working_directory: language.project_dir().to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use std::process::{ExitStatus, Output};
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    };
    use std::{fs, io};

    use aocsuite_storage::RuntimeLayout;
    use aocsuite_utils::{CommandExecutor, CommandRequest, LanguageId, ProcessMode, PuzzleYear};

    use super::{run_background_effect, run_foreground_effect, worker_loop};
    use crate::app::{
        Action, BackgroundEffect, ForegroundEffect, LanguageFileKind, PreparedLanguageFile,
    };

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

    #[test]
    fn language_list_effect_does_not_initialize_an_absent_project() {
        struct PanicExecutor;

        impl CommandExecutor for PanicExecutor {
            fn execute(&self, _: &CommandRequest) -> io::Result<Output> {
                panic!("an absent project must not run a process");
            }
        }

        let root = std::env::temp_dir().join(format!(
            "aocsuite-tui-language-list-{}-{}",
            std::process::id(),
            TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let layout = RuntimeLayout::new(root.join("runtime")).unwrap();

        let action = run_background_effect(
            &layout,
            BackgroundEffect::LoadLanguageData {
                language: LanguageId::Rust,
            },
            &PanicExecutor,
        );

        match action {
            Action::LanguageDataFinished {
                language,
                result: Ok(data),
                ..
            } => {
                assert_eq!(language, LanguageId::Rust);
                assert!(data.packages.is_empty());
                assert!(data.libraries.is_empty());
            }
            action => panic!("unexpected action: {action:?}"),
        }
        assert!(!layout.workspace_dir().exists());
        fs::remove_dir(root).unwrap();
    }

    #[test]
    fn language_file_uses_foreground_editor_request() {
        #[derive(Default)]
        struct RecordingExecutor(Mutex<Vec<CommandRequest>>);

        impl CommandExecutor for RecordingExecutor {
            fn execute(&self, request: &CommandRequest) -> io::Result<Output> {
                self.0.lock().unwrap().push(request.clone());
                Ok(Output {
                    status: successful_status(),
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
        }

        let executor = RecordingExecutor::default();
        let editor = std::env::current_exe().unwrap();
        let working_directory = std::env::temp_dir();
        let path = working_directory.join("library.rs");

        run_foreground_effect(
            ForegroundEffect::OpenLanguageFile(PreparedLanguageFile {
                language: LanguageId::Rust,
                kind: LanguageFileKind::Library("library".to_owned()),
                editor: editor.to_string_lossy().into_owned(),
                path: path.clone(),
                working_directory: working_directory.clone(),
            }),
            &executor,
        )
        .unwrap();

        let requests = executor.0.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].program, editor.into_os_string());
        assert_eq!(requests[0].args, vec![path.into_os_string()]);
        assert_eq!(requests[0].current_dir.as_ref(), Some(&working_directory));
        assert_eq!(requests[0].mode, ProcessMode::Foreground);
    }

    #[cfg(unix)]
    fn successful_status() -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatusExt::from_raw(0)
    }

    #[cfg(windows)]
    fn successful_status() -> ExitStatus {
        use std::os::windows::process::ExitStatusExt;
        ExitStatusExt::from_raw(0)
    }

    static TEST_ROOT: AtomicUsize = AtomicUsize::new(0);
}
