use aocsuite_utils::{get_aocsuite_dir, LanguageId};

use crate::{python::PythonRunner, rust::RustRunner, utils::LanguageRunner, AocLanguageResult};

pub(crate) fn to_runner(language: LanguageId) -> AocLanguageResult<LanguageRunner> {
    let root_dir = get_aocsuite_dir()?.join(language.to_string());
    let runner: LanguageRunner = match language {
        LanguageId::Rust => Box::new(RustRunner::new(root_dir)),
        LanguageId::Python => Box::new(PythonRunner::new(root_dir)),
    };
    Ok(runner)
}
