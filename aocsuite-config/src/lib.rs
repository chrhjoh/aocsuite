use std::{collections::HashMap, env, fs, path::PathBuf, str::FromStr};

use aocsuite_utils::{atomic_write, set_owner_only_permissions};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigKey {
    Language,
    Year,
    Editor,
    RunHistoryLimit,
}

impl ConfigKey {
    const ALL: [Self; 4] = [
        Self::Language,
        Self::Year,
        Self::Editor,
        Self::RunHistoryLimit,
    ];

    fn default_value(self) -> Option<&'static str> {
        match self {
            Self::RunHistoryLimit => Some("10"),
            _ => None,
        }
    }
}

impl std::fmt::Display for ConfigKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Language => "language",
            Self::Year => "year",
            Self::Editor => "editor",
            Self::RunHistoryLimit => "run_history_limit",
        })
    }
}

#[derive(Debug, Clone)]
pub struct Configuration {
    config_path: PathBuf,
    session_path: PathBuf,
    values: HashMap<ConfigKey, String>,
}

impl Configuration {
    pub fn load(config_dir: impl Into<PathBuf>) -> AocConfigResult<Self> {
        let config_dir = config_dir.into();
        let session_path = config_dir.join("session");
        let config_path = config_dir.join("config.json");

        let file_values = match fs::read(&config_path) {
            Ok(contents) => serde_json::from_slice::<HashMap<ConfigKey, String>>(&contents)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(error) => return Err(error.into()),
        };

        let mut values = ConfigKey::ALL
            .into_iter()
            .filter_map(|key| key.default_value().map(|value| (key, value.to_owned())))
            .collect::<HashMap<_, _>>();

        values.extend(file_values);

        Ok(Self {
            config_path,
            session_path: session_path.into(),
            values,
        })
    }

    pub fn get<T>(&self, key: ConfigKey) -> AocConfigResult<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let editor_fallback = if key == ConfigKey::Editor {
            match env::var("EDITOR") {
                Ok(editor) => Some(editor),
                Err(env::VarError::NotPresent) => None,
                Err(error) => return Err(AocConfigError::Environment(error)),
            }
        } else {
            None
        };
        let value = self
            .values
            .get(&key)
            .map(String::as_str)
            .or(editor_fallback.as_deref())
            .ok_or(AocConfigError::NotFound { key })?;

        value.parse::<T>().map_err(|_| AocConfigError::Invalid {
            key,
            value: value.to_owned(),
        })
    }

    pub fn set(&mut self, key: ConfigKey, value: Option<&str>) -> AocConfigResult<()> {
        let mut values = self.values.clone();
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => {
                values.insert(key, value.to_owned());
            }
            None => {
                values.remove(&key);
            }
        }

        let serialized = serde_json::to_vec_pretty(&values)?;
        atomic_write(&self.config_path, &serialized)?;
        self.values = values;

        Ok(())
    }

    pub fn session(&self) -> AocConfigResult<String> {
        Ok(fs::read_to_string(&self.session_path)?)
    }

    pub fn set_session(&self, session: Option<&str>) -> AocConfigResult<()> {
        match session.map(str::trim).filter(|value| !value.is_empty()) {
            Some(session) => {
                atomic_write(&self.session_path, session.as_bytes())?;
                set_owner_only_permissions(&self.session_path)?;
            }
            None => match fs::remove_file(&self.session_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            },
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum AocConfigError {
    #[error(transparent)]
    Environment(#[from] env::VarError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("configuration parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("invalid value '{value}' for configuration key {key}")]
    Invalid { key: ConfigKey, value: String },
    #[error("configuration key {key} was not found")]
    NotFound { key: ConfigKey },
}

pub type AocConfigResult<T> = Result<T, AocConfigError>;

#[cfg(test)]
mod tests {
    use std::fs;

    use aocsuite_utils::PuzzleYear;
    use tempfile::TempDir;

    use super::{AocConfigError, ConfigKey, Configuration};

    fn configuration(temp: &TempDir) -> Configuration {
        Configuration::load(temp.path().join("config")).unwrap()
    }

    #[test]
    fn reads_are_non_mutating_when_files_are_absent() {
        let temp = TempDir::new().unwrap();
        let config = configuration(&temp);

        assert_eq!(
            config.get::<String>(ConfigKey::RunHistoryLimit).unwrap(),
            "10"
        );
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn lowercase_file_keys_are_loaded_and_written() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config/config.json");
        fs::create_dir(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"language":"rust","year":"2024"}"#).unwrap();
        let mut config = configuration(&temp);

        assert_eq!(config.get::<String>(ConfigKey::Language).unwrap(), "rust");
        assert_eq!(
            config.get::<PuzzleYear>(ConfigKey::Year).unwrap().get(),
            2024
        );

        config.set(ConfigKey::Editor, Some("code --wait")).unwrap();
        let persisted: serde_json::Value =
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(persisted["language"], "rust");
        assert_eq!(persisted["run_history_limit"], "10");
        assert_eq!(persisted["editor"], "code --wait");
        assert!(persisted.get("Language").is_none());
    }

    #[test]
    fn malformed_config_is_preserved() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("config");
        let path = dir.join("config.json");
        let contents = br#"{"language":"rust""#;
        fs::create_dir(&dir).unwrap();
        fs::write(&path, contents).unwrap();

        assert!(matches!(
            Configuration::load(&dir),
            Err(AocConfigError::Parse(_))
        ));
        assert_eq!(fs::read(path).unwrap(), contents);
    }

    #[cfg(unix)]
    #[test]
    fn persisted_session_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("config")).unwrap();
        let config = configuration(&temp);
        config.set_session(Some("token")).unwrap();

        let mode = fs::metadata(temp.path().join("config/session"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
