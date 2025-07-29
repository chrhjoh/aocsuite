use super::RustRunner;
use crate::traits::LibManager;
use std::path::PathBuf;

impl LibManager for RustRunner {
    fn get_lib_path(&self, lib_name: &str) -> PathBuf {
        self.src_dir()
            .join(format!("{}.{}", lib_name, self.file_extention()))
    }

    fn lib_dir(&self) -> PathBuf {
        self.src_dir()
    }
    fn file_extention(&self) -> String {
        "rs".to_string()
    }

    fn invalid_lib_names(&self) -> Vec<&str> {
        return vec!["main", "solution"];
    }
}
