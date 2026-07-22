mod dependencies;
mod solver;
mod user_library;
use std::path::PathBuf;

use aocsuite_utils::CommandExecutor;

pub struct PythonRunner<'executor> {
    root_dir: PathBuf,
    executor: &'executor dyn CommandExecutor,
}
impl<'executor> PythonRunner<'executor> {
    pub fn new(root_dir: PathBuf, executor: &'executor dyn CommandExecutor) -> Self {
        Self { root_dir, executor }
    }
}
