use std::{collections::HashMap, env, fs, path::PathBuf};

use aocsuite_utils::{atomic_write, set_owner_only_permissions};
use thiserror::Error;

mod setting;

pub use setting::{ConfigKey, ConfigValue};

#[derive(Debug, Clone)]
pub struct Configuration {
    config_path: PathBuf,
    session_path: PathBuf,
    values: HashMap<ConfigKey, ConfigValue>,
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
        let values = file_values
            .into_iter()
            .map(|(key, value)| key.parse_value(value).map(|value| (key, value)))
            .collect::<AocConfigResult<HashMap<_, _>>>()?;

        Ok(Self {
            config_path,
            session_path: session_path.into(),
            values,
        })
    }

    pub fn get<T>(&self, key: ConfigKey) -> AocConfigResult<T>
    where
        T: TryFrom<ConfigValue, Error = AocConfigError>,
    {
        if key == ConfigKey::Session {
            return Err(AocConfigError::SessionReadNotAllowed);
        }

        let value = match self.values.get(&key) {
            Some(val) => val.clone(),
            None => key.default()?,
        };

        value.try_into()
    }

    pub fn set(&mut self, key: ConfigKey, value: Option<&str>) -> AocConfigResult<()> {
        if key == ConfigKey::Session {
            return self.set_session(value);
        }

        let mut values = self.values.clone();
        match value.map(str::trim).filter(|value| !value.is_empty()) {
            Some(value) => {
                values.insert(key, key.parse_value(value.to_owned())?);
            }
            None => {
                values.remove(&key);
            }
        }

        let serialized = serde_json::to_vec_pretty(
            &values
                .iter()
                .map(|(key, value)| (*key, value.to_string()))
                .collect::<HashMap<_, _>>(),
        )?;
        atomic_write(&self.config_path, &serialized)?;
        self.values = values;

        Ok(())
    }

    pub fn session(&self) -> AocConfigResult<String> {
        Ok(fs::read_to_string(&self.session_path)?)
    }

    fn set_session(&self, session: Option<&str>) -> AocConfigResult<()> {
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
    #[error("configuration value had an unexpected type")]
    UnexpectedValue,
    #[error("the session must not be stored in config.json")]
    SessionInConfig,
    #[error("reading the session configuration value is not allowed")]
    SessionReadNotAllowed,
}

pub type AocConfigResult<T> = Result<T, AocConfigError>;

#[cfg(test)]
mod tests {
    use std::fs;

    use aocsuite_utils::{LanguageId, PuzzleYear, RunHistoryLimit};
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
            config
                .get::<LanguageId>(ConfigKey::Language)
                .expect("read default language"),
            LanguageId::Rust
        );
        assert_eq!(
            config
                .get::<RunHistoryLimit>(ConfigKey::RunHistoryLimit)
                .expect("read default retention"),
            RunHistoryLimit::new(10).expect("valid default")
        );
        assert!(matches!(
            config.get::<PuzzleYear>(ConfigKey::Language),
            Err(AocConfigError::UnexpectedValue)
        ));
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn lowercase_file_keys_are_loaded_and_written() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config/config.json");
        fs::create_dir(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"language":"rust","year":"2024"}"#).unwrap();
        let mut config = configuration(&temp);

        assert_eq!(
            config
                .get::<LanguageId>(ConfigKey::Language)
                .expect("read language"),
            LanguageId::Rust
        );
        assert_eq!(
            config.get::<PuzzleYear>(ConfigKey::Year).unwrap().get(),
            2024
        );

        config.set(ConfigKey::Editor, Some("code --wait")).unwrap();
        let persisted: serde_json::Value =
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(persisted["language"], "rust");
        assert_eq!(persisted["editor"], "code --wait");
        assert!(persisted.get("run_history_limit").is_none());
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

    #[test]
    fn invalid_persisted_values_fail_during_load() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("config");
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("config.json"), r#"{"year":"invalid"}"#).unwrap();

        assert!(matches!(
            Configuration::load(dir),
            Err(AocConfigError::Invalid {
                key: ConfigKey::Year,
                ..
            })
        ));
    }

    #[test]
    fn invalid_values_are_not_written() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("config");
        fs::create_dir(&dir).unwrap();
        let mut config = Configuration::load(&dir).unwrap();

        assert!(matches!(
            config.set(ConfigKey::RunHistoryLimit, Some("0")),
            Err(AocConfigError::Invalid {
                key: ConfigKey::RunHistoryLimit,
                ..
            })
        ));
        assert!(!dir.join("config.json").exists());
    }

    #[test]
    fn effective_defaults_are_not_persisted_with_another_setting() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("config");
        fs::create_dir(&dir).unwrap();
        let mut config = Configuration::load(&dir).unwrap();

        config.set(ConfigKey::Editor, Some("vim")).unwrap();

        let persisted: serde_json::Value =
            serde_json::from_slice(&fs::read(dir.join("config.json")).unwrap()).unwrap();
        assert_eq!(persisted, serde_json::json!({ "editor": "vim" }));
        assert_eq!(
            config
                .get::<LanguageId>(ConfigKey::Language)
                .expect("read default language"),
            LanguageId::Rust
        );
        assert_eq!(
            config
                .get::<RunHistoryLimit>(ConfigKey::RunHistoryLimit)
                .expect("read default retention"),
            RunHistoryLimit::new(10).expect("valid default")
        );
    }

    #[test]
    fn session_is_rejected_from_config_json_and_cannot_be_read() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("config");
        fs::create_dir(&dir).unwrap();
        let path = dir.join("config.json");
        fs::write(&path, r#"{"session":"token"}"#).unwrap();

        assert!(matches!(
            Configuration::load(&dir),
            Err(AocConfigError::SessionInConfig)
        ));
        assert_eq!(fs::read(&path).unwrap(), br#"{"session":"token"}"#);

        fs::remove_file(path).unwrap();
        let config = Configuration::load(dir).unwrap();
        assert!(matches!(
            config.get::<String>(ConfigKey::Session),
            Err(AocConfigError::SessionReadNotAllowed)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn persisted_session_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join("config")).unwrap();
        let mut config = configuration(&temp);
        config.set(ConfigKey::Session, Some("token")).unwrap();

        let mode = fs::metadata(temp.path().join("config/session"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
