use crate::traits::LibManager;

use super::RustRunner;

impl LibManager for RustRunner {
    fn get_lib_path(&self, lib_name: &str) -> std::path::PathBuf {
        unimplemented!()
    }
    fn list_lib_files(&self) -> crate::AocLanguageResult<Vec<String>> {
        unimplemented!()
    }
    fn remove_lib_file(&self, lib_name: &str) -> crate::AocLanguageResult<()> {
        unimplemented!()
    }
}
