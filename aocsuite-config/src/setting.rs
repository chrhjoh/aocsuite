use aocsuite_utils::{LanguageId, PuzzleYear, RunHistoryLimit};

use super::{AocConfigError, AocConfigResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigKey {
    Language,
    Year,
    Editor,
    RunHistoryLimit,
    Session,
}

impl std::fmt::Display for ConfigKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Language => "language",
            Self::Year => "year",
            Self::Editor => "editor",
            Self::RunHistoryLimit => "run_history_limit",
            Self::Session => "session",
        })
    }
}

#[derive(Debug, Clone)]
pub enum ConfigValue {
    Language(LanguageId),
    Year(PuzzleYear),
    Editor(String),
    RunHistoryLimit(RunHistoryLimit),
}

impl std::fmt::Display for ConfigValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Language(value) => value.fmt(formatter),
            Self::Year(value) => value.fmt(formatter),
            Self::Editor(value) => value.fmt(formatter),
            Self::RunHistoryLimit(value) => value.fmt(formatter),
        }
    }
}

impl ConfigKey {
    pub(crate) fn parse_value(self, value: String) -> AocConfigResult<ConfigValue> {
        let invalid = || AocConfigError::Invalid {
            key: self,
            value: value.clone(),
        };

        match self {
            Self::Language => value
                .parse()
                .map(ConfigValue::Language)
                .map_err(|_| invalid()),
            Self::Year => value.parse().map(ConfigValue::Year).map_err(|_| invalid()),
            Self::Editor => {
                if value.is_empty() {
                    Err(invalid())
                } else {
                    Ok(ConfigValue::Editor(value))
                }
            }
            Self::RunHistoryLimit => value
                .parse()
                .map(ConfigValue::RunHistoryLimit)
                .map_err(|_| invalid()),
            Self::Session => Err(AocConfigError::SessionInConfig),
        }
    }

    pub(crate) fn default(self) -> AocConfigResult<ConfigValue> {
        match self {
            Self::Language => Ok(ConfigValue::Language(LanguageId::Rust)),
            Self::RunHistoryLimit => Ok(ConfigValue::RunHistoryLimit(
                RunHistoryLimit::new(10).expect("10 is a valid run history limit"),
            )),
            Self::Editor => std::env::var("EDITOR")
                .map(ConfigValue::Editor)
                .map_err(AocConfigError::from),
            _ => Err(AocConfigError::NotFound { key: self }),
        }
    }
}

macro_rules! impl_config_value_conversion {
    ($($type:ty => $variant:ident),+ $(,)?) => {
        $(
            impl TryFrom<ConfigValue> for $type {
                type Error = AocConfigError;

                fn try_from(value: ConfigValue) -> Result<Self, Self::Error> {
                    match value {
                        ConfigValue::$variant(value) => Ok(value),
                        _ => Err(AocConfigError::UnexpectedValue),
                    }
                }
            }
        )+
    };
}

impl_config_value_conversion! {
    LanguageId => Language,
    PuzzleYear => Year,
    RunHistoryLimit => RunHistoryLimit,
}

impl TryFrom<ConfigValue> for String {
    type Error = AocConfigError;

    fn try_from(value: ConfigValue) -> Result<Self, Self::Error> {
        Ok(value.to_string())
    }
}
