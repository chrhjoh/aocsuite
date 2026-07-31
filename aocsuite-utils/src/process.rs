use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

use thiserror::Error;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProcessMode {
    #[default]
    Captured,
    Foreground,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRequest {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub current_dir: Option<PathBuf>,
    pub environment: Vec<(OsString, OsString)>,
    pub inherit_environment: bool,
    pub mode: ProcessMode,
}

impl CommandRequest {
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            environment: Vec::new(),
            inherit_environment: true,
            mode: ProcessMode::Captured,
        }
    }

    pub fn arg(mut self, argument: impl Into<OsString>) -> Self {
        self.args.push(argument.into());
        self
    }

    pub fn args<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(arguments.into_iter().map(Into::into));
        self
    }

    pub fn current_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.current_dir = Some(path.into());
        self
    }

    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.environment.push((key.into(), value.into()));
        self
    }

    pub fn clear_environment(mut self) -> Self {
        self.inherit_environment = false;
        self
    }

    pub fn foreground(mut self) -> Self {
        self.mode = ProcessMode::Foreground;
        self
    }
}

pub trait CommandExecutor: Send + Sync {
    fn execute(&self, request: &CommandRequest) -> io::Result<Output>;
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("command failed: {request:?}: {output:?}")]
    Failed {
        request: Box<CommandRequest>,
        output: Box<Output>,
    },
}

pub fn execute_command(
    executor: &dyn CommandExecutor,
    request: CommandRequest,
) -> Result<Output, CommandError> {
    let output = executor.execute(&request)?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(CommandError::Failed {
            request: Box::new(request),
            output: Box::new(output),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemCommandExecutor;

impl CommandExecutor for SystemCommandExecutor {
    fn execute(&self, request: &CommandRequest) -> io::Result<Output> {
        let mut command = Command::new(&request.program);
        if !request.inherit_environment {
            command.env_clear();
        }
        command
            .args(&request.args)
            .envs(request.environment.iter().cloned());
        if let Some(current_dir) = &request.current_dir {
            command.current_dir(current_dir);
        }

        match request.mode {
            ProcessMode::Captured => command.output(),
            ProcessMode::Foreground => {
                let status = command
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()?;
                Ok(Output {
                    status,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io, sync::Mutex};

    use super::{
        execute_command, CommandError, CommandExecutor, CommandRequest, ProcessMode,
        SystemCommandExecutor,
    };

    #[derive(Default)]
    struct RecordingExecutor {
        requests: Mutex<Vec<CommandRequest>>,
    }

    impl CommandExecutor for RecordingExecutor {
        fn execute(&self, request: &CommandRequest) -> io::Result<std::process::Output> {
            self.requests.lock().unwrap().push(request.clone());
            Err(io::Error::new(io::ErrorKind::NotFound, "fake process"))
        }
    }

    #[test]
    fn requests_preserve_os_native_process_details() {
        let executor = RecordingExecutor::default();
        let request = CommandRequest::new("program")
            .arg("argument")
            .current_dir("work")
            .clear_environment()
            .env("KEY", "value")
            .foreground();

        assert!(executor.execute(&request).is_err());
        let requests = executor.requests.lock().unwrap();
        assert_eq!(requests[0], request);
        assert!(!requests[0].inherit_environment);
        assert_eq!(requests[0].mode, ProcessMode::Foreground);
    }

    #[test]
    fn system_executor_reports_launch_errors() {
        let result = SystemCommandExecutor
            .execute(&CommandRequest::new("aocsuite-command-that-must-not-exist"));

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }

    #[cfg(unix)]
    #[test]
    fn checked_execution_retains_failed_command_details() {
        use std::os::unix::process::ExitStatusExt;

        struct FailedExecutor;

        impl CommandExecutor for FailedExecutor {
            fn execute(&self, _: &CommandRequest) -> io::Result<std::process::Output> {
                Ok(std::process::Output {
                    status: std::process::ExitStatus::from_raw(1),
                    stdout: b"partial output".to_vec(),
                    stderr: b"command failed".to_vec(),
                })
            }
        }

        assert!(matches!(
            execute_command(&FailedExecutor, CommandRequest::new("failed-command")),
            Err(CommandError::Failed { output, .. })
                if output.stdout == b"partial output" && output.stderr == b"command failed"
        ));
    }
}
