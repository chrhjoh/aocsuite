use std::path::PathBuf;

use aocsuite_utils::{get_aocsuite_dir, PuzzleDay, PuzzleYear};

pub struct AocContentDir {
    base: PathBuf,
}
impl AocContentDir {
    pub fn new() -> AocContentDir {
        AocContentDir {
            base: get_aocsuite_dir().join("data"),
        }
    }

    pub fn yearly_data_dir(&self, year: PuzzleYear) -> PathBuf {
        self.base.join(format!("year{year}"))
    }
    pub fn daily_data_dir(&self, day: PuzzleDay, year: PuzzleYear) -> PathBuf {
        self.yearly_data_dir(year).join(format!("day{day}"))
    }
}
