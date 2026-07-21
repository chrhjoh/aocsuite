use std::{collections::HashMap, ffi::OsString, path::Path, process::Command};

use crate::{
    arg_builder::ArgsBuilder, editor_types::EditorCommand, AocEditorError, AocEditorResult,
};

pub struct Editor {
    program: std::path::PathBuf,
    initial_args: Vec<OsString>,
    args_builder: ArgsBuilder,
}

impl Editor {
    pub fn new(command: EditorCommand) -> Self {
        let args_builder = command.editor_type.map_or_else(
            || Box::new(crate::arg_builder::GenericArgs {}) as ArgsBuilder,
            |kind| kind.to_args_builder(),
        );
        Editor {
            program: command.program,
            initial_args: command.args,
            args_builder,
        }
    }
    fn run(
        &self,
        args: Vec<OsString>,
        env_vars: Option<HashMap<OsString, OsString>>,
    ) -> AocEditorResult<()> {
        let mut command = Command::new(&self.program);
        if let Some(vars) = env_vars {
            for (key, val) in vars.iter() {
                command.env(key, val);
            }
        }
        command.args(&self.initial_args);
        command.args(args);

        let status = command.status()?;
        if !status.success() {
            return Err(AocEditorError::RunProgram(
                self.program.display().to_string(),
            ));
        }
        Ok(())
    }

    pub fn open_solution(
        &self,
        puzzlefile: &Path,
        examplefile: &Path,
        libfile: &Path,
        inputfile: &Path,
        env_vars: Option<HashMap<OsString, OsString>>,
    ) -> AocEditorResult<()> {
        let args =
            self.args_builder
                .solution_command(puzzlefile, examplefile, libfile, inputfile)?;
        self.run(args, env_vars)?;
        Ok(())
    }
    pub fn open(
        &self,
        file: &Path,
        env_vars: Option<HashMap<OsString, OsString>>,
    ) -> AocEditorResult<()> {
        let args = vec![file.as_os_str().to_owned()];
        self.run(args, env_vars)?;
        Ok(())
    }
}
