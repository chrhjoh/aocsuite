use std::path::PathBuf;

pub struct AocCacheDir {
    base: PathBuf,
}
impl AocCacheDir {
    pub fn new(base: PathBuf) -> AocCacheDir {
        AocCacheDir { base }
    }

    pub fn puzzles_dir(&self) -> PathBuf {
        self.base.join("puzzles")
    }

    pub fn inputs_dir(&self) -> PathBuf {
        self.base.join("inputs")
    }

    pub fn calendars_dir(&self) -> PathBuf {
        self.base.join("calendars")
    }
}
