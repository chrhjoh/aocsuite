use std::{fmt, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("puzzle day must be between 1 and 25, got '{0}'")]
    PuzzleDay(String),
    #[error("puzzle year must be 2015 or later, got '{0}'")]
    PuzzleYear(String),
    #[error("invalid puzzle part '{0}'")]
    PuzzlePart(String),
    #[error("invalid part selection '{0}'")]
    PartSelection(String),
    #[error("unsupported language '{0}'")]
    Language(String),
    #[error("run history limit must be greater than zero, got '{0}'")]
    RunHistoryLimit(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PuzzleDay(u8);

impl PuzzleDay {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 25;

    pub fn new(day: u32) -> Result<Self, DomainError> {
        if (u32::from(Self::MIN)..=u32::from(Self::MAX)).contains(&day) {
            Ok(Self(day as u8))
        } else {
            Err(DomainError::PuzzleDay(day.to_string()))
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl fmt::Display for PuzzleDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for PuzzleDay {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u32>()
            .map_err(|_| DomainError::PuzzleDay(value.to_owned()))
            .and_then(Self::new)
    }
}

impl TryFrom<u32> for PuzzleDay {
    type Error = DomainError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PuzzleDay> for u32 {
    fn from(value: PuzzleDay) -> Self {
        u32::from(value.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PuzzleYear(i32);

impl PuzzleYear {
    pub const MIN: i32 = 2015;

    pub fn new(year: i32) -> Result<Self, DomainError> {
        if year >= Self::MIN {
            Ok(Self(year))
        } else {
            Err(DomainError::PuzzleYear(year.to_string()))
        }
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

impl fmt::Display for PuzzleYear {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for PuzzleYear {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<i32>()
            .map_err(|_| DomainError::PuzzleYear(value.to_owned()))
            .and_then(Self::new)
    }
}

impl TryFrom<i32> for PuzzleYear {
    type Error = DomainError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PuzzleYear> for i32 {
    fn from(value: PuzzleYear) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PuzzleId {
    pub day: PuzzleDay,
    pub year: PuzzleYear,
}

impl PuzzleId {
    pub const fn new(day: PuzzleDay, year: PuzzleYear) -> Self {
        Self { day, year }
    }
}

impl fmt::Display for PuzzleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "year{}_day{}", self.year, self.day)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PuzzlePart {
    One,
    Two,
}

impl fmt::Display for PuzzlePart {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::One => "1",
            Self::Two => "2",
        })
    }
}

impl FromStr for PuzzlePart {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "1" | "one" => Ok(Self::One),
            "2" | "two" => Ok(Self::Two),
            _ => Err(DomainError::PuzzlePart(value.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartSelection {
    One,
    Two,
    Both,
}

impl fmt::Display for PartSelection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Both => "both",
        })
    }
}

impl From<PuzzlePart> for PartSelection {
    fn from(value: PuzzlePart) -> Self {
        match value {
            PuzzlePart::One => Self::One,
            PuzzlePart::Two => Self::Two,
        }
    }
}

impl FromStr for PartSelection {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "1" | "one" => Ok(Self::One),
            "2" | "two" => Ok(Self::Two),
            "both" => Ok(Self::Both),
            _ => Err(DomainError::PartSelection(value.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageId {
    Rust,
    Python,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RunHistoryLimit(usize);

impl RunHistoryLimit {
    pub fn new(value: usize) -> Result<Self, DomainError> {
        if value == 0 {
            return Err(DomainError::RunHistoryLimit(value.to_string()));
        }
        Ok(Self(value))
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

impl fmt::Display for RunHistoryLimit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for RunHistoryLimit {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<usize>()
            .map_err(|_| DomainError::RunHistoryLimit(value.to_owned()))
            .and_then(Self::new)
    }
}

impl fmt::Display for LanguageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Rust => "rust",
            Self::Python => "python",
        })
    }
}

impl FromStr for LanguageId {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "rust" => Ok(Self::Rust),
            "python" => Ok(Self::Python),
            _ => Err(DomainError::Language(value.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LanguageId, PartSelection, PuzzleDay, PuzzleId, PuzzlePart, PuzzleYear};

    #[test]
    fn puzzle_values_validate_structural_bounds() {
        assert!(PuzzleDay::new(1).is_ok());
        assert!(PuzzleDay::new(25).is_ok());
        assert!(PuzzleDay::new(0).is_err());
        assert!(PuzzleDay::new(26).is_err());
        assert!(PuzzleYear::new(2015).is_ok());
        assert!(PuzzleYear::new(2014).is_err());

        let id = PuzzleId::new(PuzzleDay::new(4).unwrap(), PuzzleYear::new(2024).unwrap());
        assert_eq!(id.day.get(), 4);
        assert_eq!(id.year.get(), 2024);
        assert_eq!(id.to_string(), "year2024_day4");
        assert!("not-a-day".parse::<PuzzleDay>().is_err());
    }

    #[test]
    fn parts_and_languages_parse_without_frontend_traits() {
        assert_eq!("1".parse(), Ok(PuzzlePart::One));
        assert_eq!("two".parse(), Ok(PuzzlePart::Two));
        assert_eq!("both".parse(), Ok(PartSelection::Both));
        assert_eq!("RUST".parse(), Ok(LanguageId::Rust));
        assert!("three".parse::<PuzzlePart>().is_err());
        assert!("ruby".parse::<LanguageId>().is_err());
    }
}
