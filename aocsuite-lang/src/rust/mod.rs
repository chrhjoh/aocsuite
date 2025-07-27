mod solver;
use std::path::PathBuf;

pub struct RustRunner {
    root_dir: PathBuf,
}

impl RustRunner {
    pub fn new(root_dir: PathBuf) -> RustRunner {
        let runner = RustRunner { root_dir };
        std::fs::create_dir_all(runner.src_dir()).expect("is writeable");
        runner
    }
    fn src_dir(&self) -> PathBuf {
        self.root_dir.join("src")
    }
}
