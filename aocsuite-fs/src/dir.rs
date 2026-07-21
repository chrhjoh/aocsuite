use std::path::PathBuf;

use aocsuite_utils::{PuzzleDay, PuzzleYear};

pub struct AocCacheDir {
    base: PathBuf,
}
impl AocCacheDir {
    pub fn new(base: PathBuf) -> AocCacheDir {
        AocCacheDir { base }
    }

    pub fn yearly_data_dir(&self, year: PuzzleYear) -> PathBuf {
        self.base.join(format!("year{year}"))
    }
    pub fn daily_data_dir(&self, day: PuzzleDay, year: PuzzleYear) -> PathBuf {
        self.yearly_data_dir(year).join(format!("day{day}"))
    }
}
