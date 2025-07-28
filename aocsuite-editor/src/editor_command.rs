use std::process::Command;

use crate::{arg_builder::ArgsBuilder, editor_types::EditorType, AocEditorError, AocEditorResult};

pub struct EditorCommand {
    program: String,
    args_builder: ArgsBuilder,
}

impl EditorCommand {
    pub fn new(editor_type: &EditorType) -> AocEditorResult<Self> {
        let program = editor_type.to_program()?;
        let args_builder = editor_type.to_args_builder();
        Ok(EditorCommand {
            program,
            args_builder,
        })
    }
    fn run(&self, args: Vec<String>) -> AocEditorResult<()> {
        let mut command = Command::new(&self.program);
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
    ) -> AocEditorResult<()> {
        let args = self
            .args_builder
            .solution_command(puzzlefile, examplefile, libfile, inputfile);
        self.run(args)?;
        Ok(())
    }
}
