use std::path::PathBuf;

use aocsuite_utils::LanguageId;

use crate::{python::PythonRunner, rust::RustRunner, utils::LanguageRunner};

pub(crate) fn to_runner(language: LanguageId, root_dir: PathBuf) -> LanguageRunner {
    match language {
        LanguageId::Rust => Box::new(RustRunner::new(root_dir)),
        LanguageId::Python => Box::new(PythonRunner::new(root_dir)),
    }
}
