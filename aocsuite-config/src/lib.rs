use std::collections::HashMap;
use std::env::VarError;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct AocConfig {
    data: HashMap<String, String>,
    path: PathBuf,
}

impl AocConfig {
    pub fn new() -> AocConfig {
        let config_dir = PathBuf::from(".aocsuite");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).expect("Could not create config directory");
        }

        let config_path = config_dir.join("config.json");

        if !config_path.exists() {
            File::create(&config_path)
                .and_then(|mut f| f.write_all(b"{}"))
                .expect("Could not create config file");
        }

        let mut file = File::open(&config_path).expect("Could not open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Could not read config file");

        let data: HashMap<String, String> = serde_json::from_str(&contents).unwrap_or_default();

        AocConfig {
            data,
            path: config_path,
        }
    }

    pub fn get_ok(&self, key: ConfigOpt) -> AocConfigResult<String> {
        match self._get(&key) {
            Some(val) => Ok(val),
            None => Err(AocConfigError::Get { key }),
        }
    }
    pub fn get(&self, key: ConfigOpt) -> Option<String> {
        self._get(&key)
    }

    fn _get(&self, key: &ConfigOpt) -> Option<String> {
        if let Some(val) = self.data.get(&key.to_string()) {
            return Some(val.to_owned());
        }
        let env_var = match key {
            ConfigOpt::Session => "AOC_SESSION",
            ConfigOpt::Language => "AOC_LANGUAGE",
            ConfigOpt::Year => "AOC_YEAR",
            ConfigOpt::Editor => "EDITOR",
            ConfigOpt::TemplateDir => "AOC_TEMPLATE_DIR",
        };
        let val = std::env::var(env_var);
        match val {
            Ok(v) => Some(v),
            Err(_) => None,
        }
    }
    pub fn get_or_default(&self, key: ConfigOpt, default: String) -> String {
        let val = self.data.get(&key.to_string());
        match val {
            Some(val) => val.to_owned(),
            None => default,
        }
    }

    pub fn set(&mut self, key: ConfigOpt) -> AocConfigResult<()> {
        let current_value = self.get(key.clone());

        match current_value {
            Some(ref val) => print!("Enter value for {} [{}]: ", key.to_string(), val),
            None => print!("Enter value for {}: ", key.to_string()),
        }

        io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            self.data.remove(&key.to_string());
        } else {
            self.data.insert(key.to_string(), trimmed_input.to_string());
        }

        // Save to file
        let serialized =
            serde_json::to_string_pretty(&self.data).expect("Failed to serialize config");
        fs::write(&self.path, serialized).expect("Failed to write config file");

        Ok(())
    }
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
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Failed to get config key: {key:?}")]
    Get { key: ConfigOpt },
    #[error("Failed to get config key: {0}")]
    GetEnv(#[from] VarError),
}

pub type AocConfigResult<T> = Result<T, AocConfigError>;
