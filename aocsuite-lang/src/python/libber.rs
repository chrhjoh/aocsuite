use std::path::PathBuf;

use crate::traits::LibManager;

use super::PythonRunner;

impl LibManager for PythonRunner {
    fn get_lib_path(&self, lib_name: &str) -> PathBuf {
        self.root_dir
            .join(format!("{}.{}", lib_name, self.file_extention()))
    }
    fn lib_dir(&self) -> PathBuf {
        self.root_dir.clone()
    }
    fn file_extention(&self) -> String {
        "py".to_string()
    }
    fn invalid_lib_names(&self) -> Vec<&str> {
        vec!["main", "solution"]
    }
}
