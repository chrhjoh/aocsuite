use std::{
    env, fs,
    path::{Component, PathBuf},
};

use aocsuite_utils::atomic_write;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CURRENT_LAYOUT_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapReport {
    Initialized,
    Opened,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLayout {
    root: PathBuf,
}

pub fn get_aocsuite_dir() -> Result<PathBuf, LayoutError> {
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

//TODO: After implementaion of storage, and removal of fs then revise this api to not expose
// things like cache, database and workspace and other things where storage owns the files/data within.
// These files should be safe to get. Things not owned such as the language files.
impl RuntimeLayout {
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

    pub fn config_dir(&self) -> PathBuf {
        self.root.join("config")
    }

    fn layout_manifest_path(&self) -> PathBuf {
        self.root.join(".aocsuite-layout.json")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn workspace_dir(&self) -> PathBuf {
        self.root.join("workspace")
    }

    pub fn bootstrap(&self) -> Result<(), LayoutError> {
        let manifest = LayoutManifest::new();
        let manifest_path = self.layout_manifest_path();
        if !self.root.exists() {
            fs::create_dir_all(&self.root)?;
            manifest.write(&manifest_path)?
        } else if !self.root.is_dir() {
            return Err(LayoutError::RootNotDirectory(self.root.clone()));
        }
        manifest.validate(&manifest_path)?;
        self.bootstrap_directories()
    }

    fn bootstrap_directories(&self) -> Result<(), LayoutError> {
        let mut created = Vec::new();
        for directory in [self.cache_dir(), self.workspace_dir(), self.config_dir()] {
            if !directory.exists() {
                if let Err(err) = fs::create_dir(&directory) {
                    for path in created.iter().rev() {
                        let _ = fs::remove_dir(path);
                        return Err(LayoutError::Io(err));
                    }

                    created.push(directory.clone());
                }
            }
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
    fn validate(&self, path: &PathBuf) -> Result<(), LayoutError> {
        if !path.exists() {
            return Err(LayoutError::UnversionedRoot(
                path.parent().expect("manifest is not root").to_owned(),
            ));
        }

        let bytes = fs::read(path)?;
        let current_manifest: LayoutManifest = serde_json::from_slice(&bytes)?;

        if current_manifest.layout_version != self.layout_version {
            return Err(LayoutError::InvalidManifest(
                "layout_version must match".to_owned(),
            ));
        }
        if current_manifest.created_by.trim() != self.created_by.trim() {
            return Err(LayoutError::InvalidManifest(
                "created_by must match".to_owned(),
            ));
        }
        //TODO: migration logic
        Ok(())
    }
    fn write(&self, path: &PathBuf) -> Result<(), LayoutError> {
        atomic_write(path, &serde_json::to_vec_pretty(&self)?)?;
        Ok(())
    }
    fn new() -> LayoutManifest {
        LayoutManifest {
            layout_version: CURRENT_LAYOUT_VERSION,
            created_by: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
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
