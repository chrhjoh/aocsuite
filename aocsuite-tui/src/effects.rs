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
        Action, BackgroundEffect, ConfigData, ConfigMutation, ForegroundEffect, LanguageData,
        LanguageFileKind, LanguageMutation, NonSecretConfigField, PreparedExercise,
        PreparedLanguageFile,
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
        BackgroundEffect::LoadConfig { latest_year } => {
            let config_dir = layout.config_dir();
            let result = load_config_data(layout, latest_year).map_err(|error| {
                format!(
                    "Could not load configuration from '{}': {error}",
                    config_dir.display()
                )
            });
            Action::ConfigLoaded { result }
        }
        BackgroundEffect::MutateConfig {
            latest_year,
            mutation,
        } => {
            let operation = config_operation(&mutation);
            let target = match &mutation {
                ConfigMutation::SetSession(_) | ConfigMutation::RemoveSession => {
                    layout.config_dir().join("session")
                }
                ConfigMutation::Set { .. } => layout.config_dir().join("config.json"),
            };
            let result = mutate_config(layout, latest_year, mutation).map_err(|failure| {
                config_mutation_failure_message(layout, operation, &target, failure)
            });
            Action::ConfigSaved { result }
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

fn load_config_data(
    layout: &RuntimeLayout,
    latest_year: aocsuite_utils::PuzzleYear,
) -> Result<ConfigData, TuiError> {
    let config = Configuration::load(layout.config_dir())?;
    let year = match config.get::<aocsuite_utils::PuzzleYear>(ConfigKey::Year) {
        Ok(year) => year,
        Err(AocConfigError::NotFound {
            key: ConfigKey::Year,
        }) => latest_year,
        Err(error) => return Err(error.into()),
    };
    let editor = match config.get::<String>(ConfigKey::Editor) {
        Ok(editor) => Some(editor),
        Err(AocConfigError::Environment(_)) => None,
        Err(error) => return Err(error.into()),
    };
    let run_history_limit = config
        .get::<aocsuite_utils::RunHistoryLimit>(ConfigKey::RunHistoryLimit)?
        .to_string();
    Ok(ConfigData {
        year: year.to_string(),
        editor,
        run_history_limit,
        session_configured: config.session_configured()?,
    })
}

fn mutate_config(
    layout: &RuntimeLayout,
    latest_year: aocsuite_utils::PuzzleYear,
    mutation: ConfigMutation,
) -> Result<ConfigData, ConfigMutationFailure> {
    let mut config = Configuration::load(layout.config_dir())
        .map_err(|error| ConfigMutationFailure::Load(error.into()))?;
    match mutation {
        ConfigMutation::Set { field, value } => {
            let key = match field {
                NonSecretConfigField::Year => ConfigKey::Year,
                NonSecretConfigField::Editor => ConfigKey::Editor,
                NonSecretConfigField::RunHistoryLimit => ConfigKey::RunHistoryLimit,
            };
            config
                .set(key, value.as_deref())
                .map_err(|error| ConfigMutationFailure::Write(error.into()))?;
        }
        ConfigMutation::SetSession(session) => {
            config
                .set(ConfigKey::Session, Some(session.expose()))
                .map_err(|error| ConfigMutationFailure::Write(error.into()))?;
        }
        ConfigMutation::RemoveSession => config
            .set(ConfigKey::Session, None)
            .map_err(|error| ConfigMutationFailure::Write(error.into()))?,
    }
    load_config_data(layout, latest_year).map_err(ConfigMutationFailure::Reload)
}

enum ConfigMutationFailure {
    Load(TuiError),
    Write(TuiError),
    Reload(TuiError),
}

fn config_mutation_failure_message(
    layout: &RuntimeLayout,
    operation: &str,
    target: &std::path::Path,
    failure: ConfigMutationFailure,
) -> String {
    match failure {
        ConfigMutationFailure::Load(error) => format!(
            "Could not load configuration from '{}': {error}",
            layout.config_dir().join("config.json").display()
        ),
        ConfigMutationFailure::Write(error) => {
            format!("Could not {operation} at '{}': {error}", target.display())
        }
        ConfigMutationFailure::Reload(error) => format!(
            "Saved the configuration, but could not reload it from '{}': {error}",
            layout.config_dir().display()
        ),
    }
}

fn config_operation(mutation: &ConfigMutation) -> &'static str {
    match mutation {
        ConfigMutation::Set { field, .. } => match field {
            NonSecretConfigField::Year => "save the default year",
            NonSecretConfigField::Editor => "save the editor executable",
            NonSecretConfigField::RunHistoryLimit => "save run-history retention",
        },
        ConfigMutation::SetSession(_) => "save the session",
        ConfigMutation::RemoveSession => "remove the session",
    }
}

#[cfg(test)]
mod tests {
    use std::process::{ExitStatus, Output};
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    };
    use std::{fs, io};

    use aocsuite_config::{ConfigKey, Configuration};
    use aocsuite_storage::RuntimeLayout;
    use aocsuite_utils::{CommandExecutor, CommandRequest, LanguageId, ProcessMode, PuzzleYear};

    use super::{
        config_mutation_failure_message, run_background_effect, run_foreground_effect, worker_loop,
        ConfigMutationFailure,
    };
    use crate::app::{
        Action, BackgroundEffect, ConfigMutation, ForegroundEffect, LanguageFileKind,
        NonSecretConfigField, PreparedLanguageFile, SecretString,
    };

    struct PanicExecutor;

    impl CommandExecutor for PanicExecutor {
        fn execute(&self, _: &CommandRequest) -> io::Result<Output> {
            panic!("this effect must not run a process");
        }
    }

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

    #[test]
    fn config_load_is_non_mutating_and_uses_effective_defaults() {
        let root = test_root("config-load");
        let layout = RuntimeLayout::new(root.join("runtime")).unwrap();

        let action = run_background_effect(
            &layout,
            BackgroundEffect::LoadConfig {
                latest_year: PuzzleYear::new(2026).unwrap(),
            },
            &PanicExecutor,
        );

        match action {
            Action::ConfigLoaded { result: Ok(config) } => {
                assert_eq!(config.year, "2026");
                assert_eq!(config.run_history_limit, "10");
                assert!(!config.session_configured);
            }
            action => panic!("unexpected action: {action:?}"),
        }
        assert!(!layout.config_dir().exists());
        fs::remove_dir(root).unwrap();
    }

    #[test]
    fn config_mutation_preserves_other_explicit_settings() {
        let root = test_root("config-mutation");
        let layout = RuntimeLayout::new(root.join("runtime")).unwrap();
        fs::create_dir_all(layout.config_dir()).unwrap();
        let mut config = Configuration::load(layout.config_dir()).unwrap();
        config.set(ConfigKey::Language, Some("python")).unwrap();

        let action = run_background_effect(
            &layout,
            BackgroundEffect::MutateConfig {
                latest_year: PuzzleYear::new(2026).unwrap(),
                mutation: ConfigMutation::Set {
                    field: NonSecretConfigField::Year,
                    value: Some("2025".to_owned()),
                },
            },
            &PanicExecutor,
        );

        assert!(matches!(action, Action::ConfigSaved { result: Ok(_) }));
        let config = Configuration::load(layout.config_dir()).unwrap();
        assert_eq!(
            config.get::<LanguageId>(ConfigKey::Language).unwrap(),
            LanguageId::Python
        );
        assert_eq!(
            config.get::<PuzzleYear>(ConfigKey::Year).unwrap(),
            PuzzleYear::new(2025).unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn blank_config_mutations_restore_effective_defaults() {
        let root = test_root("config-reset");
        let layout = RuntimeLayout::new(root.join("runtime")).unwrap();
        fs::create_dir_all(layout.config_dir()).unwrap();
        let mut config = Configuration::load(layout.config_dir()).unwrap();
        config.set(ConfigKey::Year, Some("2025")).unwrap();
        config.set(ConfigKey::Editor, Some("vim")).unwrap();
        config.set(ConfigKey::RunHistoryLimit, Some("7")).unwrap();

        let mut last = None;
        for field in [
            NonSecretConfigField::Year,
            NonSecretConfigField::Editor,
            NonSecretConfigField::RunHistoryLimit,
        ] {
            last = Some(run_background_effect(
                &layout,
                BackgroundEffect::MutateConfig {
                    latest_year: PuzzleYear::new(2026).unwrap(),
                    mutation: ConfigMutation::Set { field, value: None },
                },
                &PanicExecutor,
            ));
        }

        assert!(matches!(
            last,
            Some(Action::ConfigSaved {
                result: Ok(crate::app::ConfigData {
                    ref year,
                    ref run_history_limit,
                    ..
                })
            }) if year == "2026" && run_history_limit == "10"
        ));
        let persisted = fs::read_to_string(layout.config_dir().join("config.json")).unwrap();
        assert!(!persisted.contains("\"year\""));
        assert!(!persisted.contains("\"editor\""));
        assert!(!persisted.contains("\"run_history_limit\""));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn session_mutations_return_only_configured_status() {
        let root = test_root("config-session");
        let layout = RuntimeLayout::new(root.join("runtime")).unwrap();
        fs::create_dir_all(layout.config_dir()).unwrap();
        let sensitive = "sensitive-value";

        let saved = run_background_effect(
            &layout,
            BackgroundEffect::MutateConfig {
                latest_year: PuzzleYear::new(2026).unwrap(),
                mutation: ConfigMutation::SetSession(SecretString::new(sensitive.to_owned())),
            },
            &PanicExecutor,
        );
        assert!(!format!("{saved:?}").contains(sensitive));
        assert!(matches!(
            saved,
            Action::ConfigSaved {
                result: Ok(crate::app::ConfigData {
                    session_configured: true,
                    ..
                })
            }
        ));

        let removed = run_background_effect(
            &layout,
            BackgroundEffect::MutateConfig {
                latest_year: PuzzleYear::new(2026).unwrap(),
                mutation: ConfigMutation::RemoveSession,
            },
            &PanicExecutor,
        );
        assert!(matches!(
            removed,
            Action::ConfigSaved {
                result: Ok(crate::app::ConfigData {
                    session_configured: false,
                    ..
                })
            }
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn config_mutation_failures_identify_phase_and_path() {
        let root = test_root("config-errors");
        let layout = RuntimeLayout::new(root.join("runtime")).unwrap();
        let target = layout.config_dir().join("session");
        let error = || {
            crate::TuiError::Io(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "permission denied",
            ))
        };

        let load = config_mutation_failure_message(
            &layout,
            "remove the session",
            &target,
            ConfigMutationFailure::Load(error()),
        );
        assert!(load.contains(
            &layout
                .config_dir()
                .join("config.json")
                .display()
                .to_string()
        ));

        let write = config_mutation_failure_message(
            &layout,
            "remove the session",
            &target,
            ConfigMutationFailure::Write(error()),
        );
        assert!(write.contains(&target.display().to_string()));

        let reload = config_mutation_failure_message(
            &layout,
            "remove the session",
            &target,
            ConfigMutationFailure::Reload(error()),
        );
        assert!(reload.contains("Saved the configuration"));
        assert!(reload.contains(&layout.config_dir().display().to_string()));
        fs::remove_dir(root).unwrap();
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

    fn test_root(label: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "aocsuite-tui-{label}-{}-{}",
            std::process::id(),
            TEST_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        root
    }
}
