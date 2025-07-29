use std::{collections::HashMap, process::Command};

use crate::{arg_builder::ArgsBuilder, editor_types::EditorType, AocEditorError, AocEditorResult};

pub struct Editor {
    program: String,
    args_builder: ArgsBuilder,
}

impl Editor {
    pub fn new(editor_type: &EditorType) -> AocEditorResult<Self> {
        let program = editor_type.to_program()?;
        let args_builder = editor_type.to_args_builder();
        Ok(Editor {
            program,
            args_builder,
        })
    }
    fn run(
        &self,
        args: Vec<String>,
        env_vars: Option<HashMap<String, String>>,
    ) -> AocEditorResult<()> {
        let mut command = Command::new(&self.program);
        if let Some(vars) = env_vars {
            for (key, val) in vars.iter() {
                command.env(key, val);
            }
        }
        command.args(args);

        let status = command.status()?;
        if !status.success() {
            return Err(AocEditorError::RunProgram(self.program.clone()));
        }
        Ok(())
    }

    pub fn open_solution(
        &self,
        puzzlefile: &str,
        examplefile: &str,
        libfile: &str,
        inputfile: &str,
        env_vars: Option<HashMap<String, String>>,
    ) -> AocEditorResult<()> {
        let args = self
            .args_builder
            .solution_command(puzzlefile, examplefile, libfile, inputfile);
        self.run(args, env_vars)?;
        Ok(())
    }
    pub fn open(
        &self,
        file: &str,
        env_vars: Option<HashMap<String, String>>,
    ) -> AocEditorResult<()> {
        let args = vec![file.to_string()];
        self.run(args, env_vars)?;
        Ok(())
    }
}
