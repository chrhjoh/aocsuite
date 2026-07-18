mod dependencies;
mod solver;
mod user_library;
use std::path::PathBuf;

pub struct RustRunner {
    root_dir: PathBuf,
}

impl RustRunner {
    pub fn new(root_dir: PathBuf) -> RustRunner {
        RustRunner { root_dir }
    }
    fn src_dir(&self) -> PathBuf {
        self.root_dir.join("src")
    }
}
