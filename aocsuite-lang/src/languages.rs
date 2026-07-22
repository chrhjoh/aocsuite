use std::path::PathBuf;

use aocsuite_utils::{CommandExecutor, LanguageId};

use crate::{python::PythonRunner, rust::RustRunner, utils::LanguageRunner};

pub(crate) fn to_runner<'executor>(
    language: LanguageId,
    project_dir: PathBuf,
    executor: &'executor dyn CommandExecutor,
) -> LanguageRunner<'executor> {
    match language {
        LanguageId::Rust => Box::new(RustRunner::new(project_dir, executor)),
        LanguageId::Python => Box::new(PythonRunner::new(project_dir, executor)),
    }
}
