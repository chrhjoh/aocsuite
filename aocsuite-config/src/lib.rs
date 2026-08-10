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
            session_path,
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

    pub fn session_configured(&self) -> AocConfigResult<bool> {
        match fs::metadata(&self.session_path) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
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

    use tempfile::TempDir;

    use super::{AocConfigError, ConfigKey, Configuration};

    fn configuration(temp: &TempDir) -> Configuration {
        Configuration::load(temp.path().join("config")).unwrap()
    }

    #[test]
    fn malformed_and_invalid_config_values_fail_during_load() {
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

        fs::write(dir.join("config.json"), r#"{"year":"invalid"}"#).unwrap();

        assert!(matches!(
            Configuration::load(&dir),
            Err(AocConfigError::Invalid {
                key: ConfigKey::Year,
                ..
            })
        ));
    }

    #[test]
    fn session_is_stored_separately_with_owner_only_permissions() {
        #[cfg(unix)]
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        let mut config = configuration(&temp);

        fs::create_dir(temp.path().join("config")).unwrap();
        config
            .set(ConfigKey::Session, Some("configured-value"))
            .unwrap();
        assert_eq!(config.session().unwrap(), "configured-value");
        assert!(!temp.path().join("config/config.json").exists());

        #[cfg(unix)]
        {
            let mode = fs::metadata(temp.path().join("config/session"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }
}
