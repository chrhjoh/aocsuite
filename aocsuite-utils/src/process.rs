use std::{
    ffi::OsString,
    io,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProcessMode {
    #[default]
    Captured,
    Foreground,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRequest {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub current_dir: Option<PathBuf>,
    pub environment: Vec<(OsString, OsString)>,
    pub inherit_environment: bool,
    pub mode: ProcessMode,
}

impl ProcessRequest {
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

pub trait ProcessExecutor: Send + Sync {
    fn execute(&self, request: &ProcessRequest) -> io::Result<Output>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemProcessExecutor;

impl ProcessExecutor for SystemProcessExecutor {
    fn execute(&self, request: &ProcessRequest) -> io::Result<Output> {
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

    use super::{ProcessExecutor, ProcessMode, ProcessRequest, SystemProcessExecutor};

    #[derive(Default)]
    struct RecordingExecutor {
        requests: Mutex<Vec<ProcessRequest>>,
    }

    impl ProcessExecutor for RecordingExecutor {
        fn execute(&self, request: &ProcessRequest) -> io::Result<std::process::Output> {
            self.requests.lock().unwrap().push(request.clone());
            Err(io::Error::new(io::ErrorKind::NotFound, "fake process"))
        }
    }

    #[test]
    fn requests_preserve_os_native_process_details() {
        let executor = RecordingExecutor::default();
        let request = ProcessRequest::new("program")
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
        let result = SystemProcessExecutor
            .execute(&ProcessRequest::new("aocsuite-command-that-must-not-exist"));

        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
    }
}
