use std::collections::HashMap;
use std::env::VarError;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use aocsuite_utils::{get_aocsuite_dir, RuntimeDirError};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct AocConfig {
    data: HashMap<String, String>,
    path: PathBuf,
}

impl AocConfig {
    pub fn new() -> AocConfigResult<AocConfig> {
        let config_dir = get_aocsuite_dir()?;

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join("config.json");

        if !config_path.exists() {
            fs::write(&config_path, b"{}")?;
        }

        let contents = fs::read(&config_path)?;
        let data = serde_json::from_slice(&contents)?;

        Ok(AocConfig {
            data,
            path: config_path,
        })
    }
    pub fn get(&self, key: &ConfigOpt) -> Option<String> {
        if let Some(val) = self.data.get(&key.to_string()) {
            return Some(val.to_owned());
        }
        let env_var = match key {
            ConfigOpt::Session => "AOC_SESSION",
            ConfigOpt::Language => "AOC_LANGUAGE",
            ConfigOpt::Year => "AOC_YEAR",
            ConfigOpt::Editor => "AOC_EDITOR",
            ConfigOpt::TemplateDir => "AOC_TEMPLATE_DIR",
        };
        let val = std::env::var(env_var);
        match val {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
    pub fn set(&mut self, key: &ConfigOpt) -> AocConfigResult<()> {
        let current_value = self.get(key);

        match current_value {
            Some(ref val) => print!("Enter value for {} [{}]: ", key.to_string(), val),
            None => print!("Enter value for {}: ", key.to_string()),
        }

        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            self.data.remove(&key.to_string());
        } else {
            self.data.insert(key.to_string(), trimmed_input.to_string());
        }

        // Save to file
        let serialized = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, serialized)?;

        Ok(())
    }
}

pub fn get_config_val<T>(
    key: &ConfigOpt,
    default: Option<T>,
    overwrite: Option<T>,
) -> AocConfigResult<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    if let Some(val) = overwrite {
        return Ok(val);
    }

    let config = AocConfig::new()?;
    if let Some(val) = config.get(key) {
        return Ok(val.parse().map_err(|_| AocConfigError::Invalid {
            key: key.clone(),
            val,
        })?);
    }

    if let Some(val) = default {
        return Ok(val);
    }
    Err(AocConfigError::NotFound { key: key.clone() })
}

pub fn set_config_val(key: &ConfigOpt) -> AocConfigResult<()> {
    let mut config = AocConfig::new()?;
    config.set(key)
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ConfigOpt {
    Language,
    Year,
    Editor,
    Session,
    TemplateDir,
}

impl ToString for ConfigOpt {
    fn to_string(&self) -> String {
        match self {
            ConfigOpt::Language => "language",
            ConfigOpt::Year => "year",
            ConfigOpt::Editor => "editor",
            ConfigOpt::Session => "session",
            ConfigOpt::TemplateDir => "template_dir",
        }
        .to_string()
    }
}

#[derive(Debug, Error)]
pub enum AocConfigError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Failed to get config key: {key:?}. Invalid value: {val})")]
    Invalid { key: ConfigOpt, val: String },
    #[error("Failed to get config key: {key:?} Not Found")]
    NotFound { key: ConfigOpt },
    #[error("Failed to get config key: {0}")]
    GetEnv(#[from] VarError),
    #[error(transparent)]
    RuntimeDir(#[from] RuntimeDirError),
}

pub type AocConfigResult<T> = Result<T, AocConfigError>;

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{get_config_val, set_config_val, AocConfigError, ConfigOpt};

    #[test]
    fn malformed_config_is_returned_without_being_overwritten() {
        let data_home = std::env::temp_dir().join(format!(
            "aocsuite-config-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before Unix epoch")
                .as_nanos()
        ));
        let config_dir = data_home.join("aocsuite");
        let config_path = config_dir.join("config.json");
        let contents = br#"{"language":"rust""#;
        fs::create_dir_all(&config_dir).expect("create config directory");
        fs::write(&config_path, contents).expect("write malformed config");

        let previous_data_home = std::env::var_os("XDG_DATA_HOME");
        std::env::set_var("XDG_DATA_HOME", &data_home);

        assert!(matches!(
            get_config_val::<String>(&ConfigOpt::Language, None, None),
            Err(AocConfigError::Parse(_))
        ));
        assert!(matches!(
            set_config_val(&ConfigOpt::Language),
            Err(AocConfigError::Parse(_))
        ));
        assert_eq!(fs::read(&config_path).expect("read config"), contents);

        match previous_data_home {
            Some(value) => std::env::set_var("XDG_DATA_HOME", value),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        fs::remove_dir_all(data_home).expect("remove test data directory");
    }
}
