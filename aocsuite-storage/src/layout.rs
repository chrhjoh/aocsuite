use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use aocsuite_utils::{
    atomic_write, get_aocsuite_dir, LanguageId, PuzzleId, PuzzleYear, RuntimeDirError,
};
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

impl RuntimeLayout {
    pub fn discover() -> Result<Self, LayoutError> {
        Self::new(get_aocsuite_dir()?)
    }

    pub fn new(root: impl Into<PathBuf>) -> Result<Self, LayoutError> {
        let root = root.into();
        if root.as_os_str().is_empty()
            || !root.is_absolute()
            || root.components().any(|part| part == Component::ParentDir)
        {
            return Err(LayoutError::InvalidRoot(root));
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> PathBuf {
        self.root.join(LAYOUT_MANIFEST)
    }

    pub fn preferences(&self) -> PathBuf {
        self.root.join("config.json")
    }

    pub fn secrets(&self) -> PathBuf {
        self.root.join("secrets")
    }

    pub fn session(&self) -> PathBuf {
        self.secrets().join("session")
    }

    pub fn database(&self) -> PathBuf {
        self.root.join("state.sqlite")
    }

    pub fn cache(&self) -> PathBuf {
        self.root.join("cache").join("aoc")
    }

    pub fn runs(&self) -> PathBuf {
        self.root.join("runs")
    }

    pub fn workspace(&self) -> PathBuf {
        self.root.join("workspace")
    }

    pub fn examples(&self) -> PathBuf {
        self.workspace().join("examples")
    }

    pub fn example(&self, puzzle: PuzzleId) -> PathBuf {
        self.examples()
            .join(format!("year{}", puzzle.year))
            .join(format!("day{}.txt", puzzle.day))
    }

    pub fn language_project(&self, language: LanguageId) -> PathBuf {
        self.workspace().join(language.to_string())
    }

    pub fn cache_path(&self, key: CacheKey) -> PathBuf {
        match key {
            CacheKey::PuzzleHtml(puzzle) => self.puzzle_cache_dir(puzzle).join("puzzle.html"),
            CacheKey::PuzzleMarkdown(puzzle) => self.puzzle_cache_dir(puzzle).join("puzzle.md"),
            CacheKey::Input(puzzle) => self.puzzle_cache_dir(puzzle).join("input.txt"),
            CacheKey::Calendar(year) => self
                .cache()
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

        let manifest = self.manifest();
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
        self.cache()
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
            set_owner_only_directory(&self.root)?;

            for directory in [
                self.secrets(),
                self.root.join("cache"),
                self.cache(),
                self.runs(),
            ] {
                if !directory.exists() {
                    fs::create_dir(&directory)?;
                    created.push(directory.clone());
                }
                set_owner_only_directory(&directory)?;
            }

            let manifest = LayoutManifest {
                layout_version: CURRENT_LAYOUT_VERSION,
                created_by: env!("CARGO_PKG_VERSION").to_owned(),
            };
            atomic_write(&self.manifest(), &serde_json::to_vec_pretty(&manifest)?)?;
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
        set_owner_only_directory(&self.root)?;
        for directory in [
            self.secrets(),
            self.root.join("cache"),
            self.cache(),
            self.runs(),
        ] {
            fs::create_dir_all(&directory)?;
            set_owner_only_directory(&directory)?;
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

fn set_owner_only_directory(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
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
    #[error(transparent)]
    RuntimeDir(#[from] RuntimeDirError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use aocsuite_utils::{LanguageId, PuzzleDay, PuzzleId, PuzzleYear};
    use tempfile::TempDir;

    use super::{BootstrapReport, CacheKey, LayoutError, RuntimeLayout, CURRENT_LAYOUT_VERSION};

    fn layout(temp: &TempDir) -> RuntimeLayout {
        RuntimeLayout::new(temp.path().join("aocsuite")).expect("valid explicit root")
    }

    fn puzzle() -> PuzzleId {
        PuzzleId::new(
            PuzzleDay::new(4).expect("valid test day"),
            PuzzleYear::new(2024).expect("valid test year"),
        )
    }

    #[test]
    fn path_getters_are_pure() {
        let temp = TempDir::new().expect("create temporary root");
        let layout = layout(&temp);
        let puzzle = puzzle();

        assert_eq!(layout.preferences(), layout.root().join("config.json"));
        assert_eq!(layout.session(), layout.root().join("secrets/session"));
        assert_eq!(layout.database(), layout.root().join("state.sqlite"));
        assert_eq!(layout.runs(), layout.root().join("runs"));
        assert_eq!(
            layout.example(puzzle),
            layout.root().join("workspace/examples/year2024/day4.txt")
        );
        assert_eq!(
            layout.language_project(LanguageId::Rust),
            layout.root().join("workspace/rust")
        );
        assert_eq!(
            layout.cache_path(CacheKey::PuzzleHtml(puzzle)),
            layout.root().join("cache/aoc/year2024/day4/puzzle.html")
        );
        assert_eq!(
            layout.cache_path(CacheKey::PuzzleMarkdown(puzzle)),
            layout.root().join("cache/aoc/year2024/day4/puzzle.md")
        );
        assert_eq!(
            layout.cache_path(CacheKey::Input(puzzle)),
            layout.root().join("cache/aoc/year2024/day4/input.txt")
        );
        assert_eq!(
            layout.cache_path(CacheKey::Calendar(puzzle.year)),
            layout.root().join("cache/aoc/year2024/calendar.html")
        );
        assert!(!layout.root().exists());
    }

    #[test]
    fn missing_and_empty_roots_initialize_layout_one_without_workspace() {
        for precreate in [false, true] {
            let temp = TempDir::new().expect("create temporary root");
            let layout = layout(&temp);
            if precreate {
                fs::create_dir(layout.root()).expect("create empty root");
            }

            assert_eq!(layout.bootstrap().unwrap(), BootstrapReport::Initialized);
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(layout.manifest()).unwrap()).unwrap();
            assert_eq!(manifest["layout_version"], CURRENT_LAYOUT_VERSION);
            assert!(layout.secrets().is_dir());
            assert!(layout.cache().is_dir());
            assert!(layout.runs().is_dir());
            assert!(!layout.workspace().exists());
            assert!(!layout.preferences().exists());
            assert!(!layout.database().exists());
        }
    }

    #[test]
    fn current_layout_reopens_and_repairs_owned_directories() {
        let temp = TempDir::new().expect("create temporary root");
        let layout = layout(&temp);
        layout.bootstrap().unwrap();
        fs::remove_dir_all(layout.cache()).unwrap();

        assert_eq!(layout.bootstrap().unwrap(), BootstrapReport::Opened);
        assert!(layout.cache().is_dir());
        assert!(!layout.workspace().exists());
    }

    #[test]
    fn nonempty_unversioned_root_is_rejected_without_mutation() {
        let temp = TempDir::new().expect("create temporary root");
        let layout = layout(&temp);
        fs::create_dir(layout.root()).unwrap();
        let sentinel = layout.root().join("keep-me");
        fs::write(&sentinel, "unchanged").unwrap();

        assert!(matches!(
            layout.bootstrap(),
            Err(LayoutError::UnversionedRoot(_))
        ));
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "unchanged");
        assert!(!layout.secrets().exists());
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
            let temp = TempDir::new().expect("create temporary root");
            let layout = layout(&temp);
            fs::create_dir(layout.root()).unwrap();
            fs::write(layout.manifest(), &manifest).unwrap();

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
            assert_eq!(fs::read_to_string(layout.manifest()).unwrap(), manifest);
            assert!(!layout.secrets().exists());
        }
    }

    #[test]
    fn explicit_roots_must_be_absolute() {
        assert!(matches!(
            RuntimeLayout::new("relative"),
            Err(LayoutError::InvalidRoot(_))
        ));
        let temp = TempDir::new().expect("create temporary root");
        assert!(matches!(
            RuntimeLayout::new(temp.path().join("aocsuite/../escape")),
            Err(LayoutError::InvalidRoot(_))
        ));
    }

    #[test]
    fn runtime_root_must_be_a_directory() {
        let temp = TempDir::new().expect("create temporary root");
        let layout = layout(&temp);
        fs::write(layout.root(), "not a directory").unwrap();

        assert!(matches!(
            layout.bootstrap(),
            Err(LayoutError::RootNotDirectory(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn runtime_and_secret_directories_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().expect("create temporary root");
        let layout = layout(&temp);
        layout.bootstrap().unwrap();

        for path in [layout.root().to_path_buf(), layout.secrets()] {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }
}
