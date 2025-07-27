mod solver;
use std::path::PathBuf;

pub struct PythonRunner {
    root_dir: PathBuf,
}
impl PythonRunner {
    pub fn new(root_dir: PathBuf) -> PythonRunner {
        PythonRunner { root_dir }
    }
}
