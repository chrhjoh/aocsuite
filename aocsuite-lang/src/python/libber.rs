use crate::traits::LibManager;

use super::PythonRunner;

impl LibManager for PythonRunner {
    fn get_lib_path(&self, lib_name: &str) -> std::path::PathBuf {
        unimplemented!()
    }
    fn remove_lib_file(&self, lib_name: &str) -> crate::AocLanguageResult<()> {
        unimplemented!()
    }
    fn list_lib_files(&self) -> crate::AocLanguageResult<Vec<String>> {
        unimplemented!()
    }
}
