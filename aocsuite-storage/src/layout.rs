use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

use aocsuite_utils::{atomic_write, LanguageId, PuzzleId, PuzzleYear};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CURRENT_LAYOUT_VERSION: u32 = 1;
const LAYOUT_MANIFEST: &str = ".aocsuite-layout.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapReport {
    Initialized,
    Opened,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheKey {
    PuzzleHtml(PuzzleId),
    PuzzleMarkdown(PuzzleId),
    Input(PuzzleId),
    Calendar(PuzzleYear),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLayout {
    root: PathBuf,
}

fn get_aocsuite_dir() -> Result<PathBuf, LayoutError> {
    let environment_preferences = [
        (
            "AOCSUITE_DATA_DIR",
            env::var_os("AOCSUITE_DATA_DIR"),
            PathBuf::new(),
        ),
        (
            "XDG_DATA_HOME",
            env::var_os("XDG_DATA_HOME"),
            PathBuf::from("aocsuite"),
        ),
        (
            "HOME",
            env::var_os("HOME"),
            PathBuf::from(".local/share/aocsuite"),
        ),
    ];

    for (variable, value, suffix) in environment_preferences {
        let Some(value) = value else {
            continue;
        };

        let base_path = PathBuf::from(value);

        if !valid_environment_path(&base_path) {
            return Err(LayoutError::InvalidEnvironmentPath {
                variable,
                path: base_path,
            });
        }

        return Ok(base_path.join(suffix));
    }

    Err(LayoutError::MissingHome)
}

fn valid_environment_path(path: &PathBuf) -> bool {
    !path.as_os_str().is_empty() && path.is_absolute()
}

impl RuntimeLayout {
    pub fn new() -> Result<Self, LayoutError> {
        let root = get_aocsuite_dir()?;
        if root.as_os_str().is_empty()
            || !root.is_absolute()
            || root.components().any(|part| part == Component::ParentDir)
        {
            return Err(LayoutError::InvalidRoot(root));
        }
        Ok(Self { root })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root
    }

    pub fn layout_manifest_path(&self) -> PathBuf {
        self.root.join(LAYOUT_MANIFEST)
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.json")
    }

    pub fn session_path(&self) -> PathBuf {
        self.root.join("session")
    }

    pub fn database_path(&self) -> PathBuf {
        self.root.join("state.sqlite")
    }

    pub fn aoc_cache_dir(&self) -> PathBuf {
        self.root.join("cache").join("aoc")
    }

    pub fn runs_dir(&self) -> PathBuf {
        self.root.join("runs")
    }

    pub fn workspace_dir(&self) -> PathBuf {
        self.root.join("workspace")
    }

    pub fn examples_dir(&self) -> PathBuf {
        self.workspace_dir().join("examples")
    }

    pub fn example_path(&self, puzzle: PuzzleId) -> PathBuf {
        self.examples_dir()
            .join(format!("year{}", puzzle.year))
            .join(format!("day{}.txt", puzzle.day))
    }

    pub fn language_project_dir(&self, language: LanguageId) -> PathBuf {
        self.workspace_dir().join(language.to_string())
    }

    pub fn cache_path(&self, key: CacheKey) -> PathBuf {
        match key {
            CacheKey::PuzzleHtml(puzzle) => self.puzzle_cache_dir(puzzle).join("puzzle.html"),
            CacheKey::PuzzleMarkdown(puzzle) => self.puzzle_cache_dir(puzzle).join("puzzle.md"),
            CacheKey::Input(puzzle) => self.puzzle_cache_dir(puzzle).join("input.txt"),
            CacheKey::Calendar(year) => self
                .aoc_cache_dir()
                .join(format!("year{year}"))
                .join("calendar.html"),
        }
    }

    pub fn bootstrap(&self) -> Result<BootstrapReport, LayoutError> {
        match fs::metadata(&self.root) {
            Ok(metadata) if !metadata.is_dir() => {
                return Err(LayoutError::RootNotDirectory(self.root.clone()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return self.initialize(true);
            }
            Err(error) => return Err(error.into()),
        }

        let manifest = self.layout_manifest_path();
        if !manifest.exists() {
            if directory_is_empty(&self.root)? {
                return self.initialize(false);
            }
            return Err(LayoutError::UnversionedRoot(self.root.clone()));
        }

        let bytes = fs::read(&manifest)?;
        let manifest: LayoutManifest = serde_json::from_slice(&bytes)?;
        manifest.validate()?;
        if manifest.layout_version != CURRENT_LAYOUT_VERSION {
            return Err(LayoutError::UnsupportedLayoutVersion {
                found: manifest.layout_version,
                supported: CURRENT_LAYOUT_VERSION,
            });
        }

        self.ensure_owned_directories()?;
        Ok(BootstrapReport::Opened)
    }

    fn puzzle_cache_dir(&self, puzzle: PuzzleId) -> PathBuf {
        self.aoc_cache_dir()
            .join(format!("year{}", puzzle.year))
            .join(format!("day{}", puzzle.day))
    }

    fn initialize(&self, create_root: bool) -> Result<BootstrapReport, LayoutError> {
        let mut created = Vec::new();
        let result = (|| {
            if create_root {
                let mut missing_ancestors = self
                    .root
                    .ancestors()
                    .take_while(|path| !path.exists())
                    .map(Path::to_path_buf)
                    .collect::<Vec<_>>();
                missing_ancestors.reverse();
                created.extend(missing_ancestors);
                fs::create_dir_all(&self.root)?;
            }

            for directory in [
                self.root.join("cache"),
                self.aoc_cache_dir(),
                self.runs_dir(),
            ] {
                if !directory.exists() {
                    fs::create_dir(&directory)?;
                    created.push(directory.clone());
                }
            }

            let manifest = LayoutManifest {
                layout_version: CURRENT_LAYOUT_VERSION,
                created_by: env!("CARGO_PKG_VERSION").to_owned(),
            };
            atomic_write(
                &self.layout_manifest_path(),
                &serde_json::to_vec_pretty(&manifest)?,
            )?;
            Ok(BootstrapReport::Initialized)
        })();

        if result.is_err() {
            for path in created.iter().rev() {
                let _ = fs::remove_dir(path);
            }
        }
        result
    }

    fn ensure_owned_directories(&self) -> Result<(), LayoutError> {
        for directory in [
            self.root.join("cache"),
            self.aoc_cache_dir(),
            self.runs_dir(),
        ] {
            fs::create_dir_all(&directory)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutManifest {
    layout_version: u32,
    created_by: String,
}

impl LayoutManifest {
    fn validate(&self) -> Result<(), LayoutError> {
        if self.layout_version == 0 {
            return Err(LayoutError::InvalidManifest(
                "layout_version must be greater than zero".to_owned(),
            ));
        }
        if self.created_by.trim().is_empty() {
            return Err(LayoutError::InvalidManifest(
                "created_by must not be empty".to_owned(),
            ));
        }
        Ok(())
    }
}

fn directory_is_empty(path: &Path) -> Result<bool, std::io::Error> {
    Ok(fs::read_dir(path)?.next().transpose()?.is_none())
}

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("runtime root must be an absolute, non-empty path: {0}")]
    InvalidRoot(PathBuf),
    #[error("runtime root is not a directory: {0}")]
    RootNotDirectory(PathBuf),
    #[error(
        "runtime root is nonempty and has no layout manifest: {0}; remove it manually before initializing AoC Suite"
    )]
    UnversionedRoot(PathBuf),
    #[error("unsupported layout version {found}; this binary supports version {supported}")]
    UnsupportedLayoutVersion { found: u32, supported: u32 },
    #[error("invalid layout manifest: {0}")]
    InvalidManifest(String),
    #[error("{variable} must contain an absolute, non-empty path: {path:?}")]
    InvalidEnvironmentPath {
        variable: &'static str,
        path: PathBuf,
    },
    #[error("none of AOCSUITE_DATA_DIR, XDG_DATA_HOME, or HOME are set")]
    MissingHome,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzleYear};

    use super::{BootstrapReport, CacheKey, LayoutError, RuntimeLayout, CURRENT_LAYOUT_VERSION};

    fn puzzle() -> PuzzleId {
        PuzzleId::new(
            PuzzleDay::new(4).expect("valid test day"),
            PuzzleYear::new(2024).expect("valid test year"),
        )
    }

    #[test]
    fn path_getters_are_pure() {
        //TODO:  allocate a dir with AOCSUITE_DATA_DIR
        let layout = RuntimeLayout::new().expect("valid explicit root");

        let puzzle = puzzle();

        assert_eq!(layout.config_path(), layout.root_dir().join("config.json"));
        assert_eq!(layout.session_path(), layout.root_dir().join("session"));
        assert_eq!(
            layout.database_path(),
            layout.root_dir().join("state.sqlite")
        );
        assert_eq!(layout.runs_dir(), layout.root_dir().join("runs"));
        assert_eq!(
            layout.example_path(puzzle),
            layout
                .root_dir()
                .join("workspace/examples/year2024/day4.txt")
        );
        assert_eq!(
            layout.language_project_dir(LanguageId::Rust),
            layout.root_dir().join("workspace/rust")
        );
        assert_eq!(
            layout.cache_path(CacheKey::PuzzleHtml(puzzle)),
            layout
                .root_dir()
                .join("cache/aoc/year2024/day4/puzzle.html")
        );
        assert_eq!(
            layout.cache_path(CacheKey::PuzzleMarkdown(puzzle)),
            layout.root_dir().join("cache/aoc/year2024/day4/puzzle.md")
        );
        assert_eq!(
            layout.cache_path(CacheKey::Input(puzzle)),
            layout.root_dir().join("cache/aoc/year2024/day4/input.txt")
        );
        assert_eq!(
            layout.cache_path(CacheKey::Calendar(puzzle.year)),
            layout.root_dir().join("cache/aoc/year2024/calendar.html")
        );
        assert!(!layout.root_dir().exists());
    }

    #[test]
    fn missing_and_empty_roots_initialize_layout_one_without_workspace() {
        for precreate in [false, true] {
            let layout = RuntimeLayout::new().expect("valid explicit root");
            if precreate {
                fs::create_dir(layout.root_dir()).expect("create empty root");
            }

            assert_eq!(layout.bootstrap().unwrap(), BootstrapReport::Initialized);
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(layout.layout_manifest_path()).unwrap()).unwrap();
            assert_eq!(manifest["layout_version"], CURRENT_LAYOUT_VERSION);
            assert!(layout.aoc_cache_dir().is_dir());
            assert!(layout.runs_dir().is_dir());
            assert!(!layout.workspace_dir().exists());
            assert!(!layout.config_path().exists());
            assert!(!layout.database_path().exists());
        }
    }

    #[test]
    fn current_layout_reopens_and_repairs_owned_directories() {
        let layout = RuntimeLayout::new().expect("valid explicit root");
        layout.bootstrap().unwrap();
        fs::remove_dir_all(layout.aoc_cache_dir()).unwrap();

        assert_eq!(layout.bootstrap().unwrap(), BootstrapReport::Opened);
        assert!(layout.aoc_cache_dir().is_dir());
        assert!(!layout.workspace_dir().exists());
    }

    #[test]
    fn nonempty_unversioned_root_is_rejected_without_mutation() {
        let layout = RuntimeLayout::new().expect("valid explicit root");
        fs::create_dir(layout.root_dir()).unwrap();
        let sentinel = layout.root_dir().join("keep-me");
        fs::write(&sentinel, "unchanged").unwrap();

        assert!(matches!(
            layout.bootstrap(),
            Err(LayoutError::UnversionedRoot(_))
        ));
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "unchanged");
    }

    #[test]
    fn malformed_and_unsupported_manifests_are_rejected_without_mutation() {
        for (manifest, expected) in [
            ("not json".to_owned(), "json"),
            (
                format!(
                    "{{\"layout_version\":{},\"created_by\":\"future\"}}",
                    CURRENT_LAYOUT_VERSION + 1
                ),
                "unsupported",
            ),
            (
                "{\"layout_version\":1,\"created_by\":\"\"}".to_owned(),
                "invalid",
            ),
        ] {
            let layout = RuntimeLayout::new().expect("valid explicit root");
            fs::create_dir(layout.root_dir()).unwrap();
            fs::write(layout.layout_manifest_path(), &manifest).unwrap();

            let error = layout.bootstrap().unwrap_err();
            match expected {
                "json" => assert!(matches!(error, LayoutError::Json(_))),
                "unsupported" => assert!(matches!(
                    error,
                    LayoutError::UnsupportedLayoutVersion { .. }
                )),
                "invalid" => assert!(matches!(error, LayoutError::InvalidManifest(_))),
                _ => unreachable!(),
            }
            assert_eq!(
                fs::read_to_string(layout.layout_manifest_path()).unwrap(),
                manifest
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn runtime_and_secret_directories_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let layout = RuntimeLayout::new().expect("valid explicit root");
        layout.bootstrap().unwrap();

        for path in [layout.root_dir().to_path_buf()] {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }
}
