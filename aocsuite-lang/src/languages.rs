use std::path::PathBuf;

use aocsuite_utils::LanguageId;

use crate::{python::PythonRunner, rust::RustRunner, utils::LanguageRunner};

pub(crate) fn to_runner(language: LanguageId, project_dir: PathBuf) -> LanguageRunner {
    match language {
        LanguageId::Rust => Box::new(RustRunner::new(project_dir)),
        LanguageId::Python => Box::new(PythonRunner::new(project_dir)),
    }
}
