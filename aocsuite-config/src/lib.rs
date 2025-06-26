use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
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

    pub fn get(&self, key: ConfigOpt) -> AocConfigResult<&str> {
        match self.data.get(&key.to_string()) {
            Some(val) => Ok(val),
            None => Err(AocConfigError::Get { key }),
        }
    }
    pub fn get_or_default(&self, key: ConfigOpt, default: String) -> String {
        let val = self.data.get(&key.to_string());
        match val {
            Some(val) => val.to_owned(),
            None => default,
        }
    }

    pub fn set(&mut self, key: ConfigOpt, value: String) -> AocConfigResult<()> {
        self.data.insert(key.to_string(), value);
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
}

impl ToString for ConfigOpt {
    fn to_string(&self) -> String {
        match self {
            ConfigOpt::Language => "language",
            ConfigOpt::Year => "year",
            ConfigOpt::Editor => "editor",
            ConfigOpt::Session => "session",
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
}

pub type AocConfigResult<T> = Result<T, AocConfigError>;
