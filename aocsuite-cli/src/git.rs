use std::path::{Path, PathBuf};

use aocsuite_utils::RuntimeDirError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AocGitError {
    #[error("Git command exited with code {code}: {stderr}")]
    CommandFailed { code: i32, stderr: String },

    #[error("Failed to resolve aocsuite directory")]
    DirectoryNotFound,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    RuntimeDir(#[from] RuntimeDirError),

    #[error("Only clone in format `git clone my_git_repo` is supported")]
    Clone,
}

pub type AocGitResult<T> = Result<T, AocGitError>;

pub fn get_gitignore_path() -> AocGitResult<PathBuf> {
    let aocsuite_dir = aocsuite_utils::get_aocsuite_dir()?;
    std::fs::create_dir_all(&aocsuite_dir)?;
    let path = aocsuite_dir.join(".gitignore");
    ensure_gitignore_exists(&path)?;
    Ok(path)
}

fn ensure_gitignore_exists(path: &Path) -> AocGitResult<()> {
    if !path.exists() {
        let default_content = get_default_gitignore_content();
        std::fs::write(path, default_content)?;
    }
    Ok(())
}

fn get_default_gitignore_content() -> &'static str {
    r#"# AOC Suite generated files
data/
config.json

# Language specific
# Rust
Cargo.lock
target/

# Python
__pycache__/

"#
}

pub fn run_git_command(args: &[String]) -> AocGitResult<String> {
    let aocsuite_dir = aocsuite_utils::get_aocsuite_dir()?;
    run_git_command_with(args, &aocsuite_dir)
}

fn run_git_command_with(args: &[String], aocsuite_dir: &Path) -> AocGitResult<String> {
    if is_simple_clone(args)? {
        let parent = aocsuite_dir
            .parent()
            .ok_or(AocGitError::DirectoryNotFound)?;
        std::fs::create_dir_all(parent)?;

        let destination = aocsuite_dir
            .file_name()
            .ok_or(AocGitError::DirectoryNotFound)?
            .to_string_lossy()
            .into_owned();
        let mut clone_args = args.to_vec();
        clone_args.push(destination);
        return run_git_command_capture(&clone_args, parent);
    }

    std::fs::create_dir_all(aocsuite_dir)?;
    ensure_gitignore_exists(&aocsuite_dir.join(".gitignore"))?;
    if is_interactive_command(args) {
        run_git_command_interactive(args, aocsuite_dir)
    } else {
        run_git_command_capture(args, aocsuite_dir)
    }
}

fn run_git_command_capture(args: &[String], dir: &Path) -> AocGitResult<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Commands like git help exit unsuccessfully but is not an error
        if stderr.is_empty() {
            let message = String::from_utf8_lossy(&output.stdout).to_string();
            if !message.is_empty() {
                return Ok(message);
            }
        }
        return Err(AocGitError::CommandFailed { code, stderr });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_git_command_interactive(args: &[String], dir: &Path) -> AocGitResult<String> {
    let status = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    if !status.success() {
        return Err(AocGitError::CommandFailed {
            code: status.code().unwrap_or(1),
            stderr: "interactive git command failed".to_string(),
        });
    }

    Ok("".to_string())
}
fn is_interactive_command(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }

    match args[0].as_str() {
        "commit" => {
            // Check if -m or --message is provided
            !args
                .iter()
                .any(|arg| arg == "-m" || arg.starts_with("--message"))
        }
        "rebase" => {
            // Interactive rebase
            args.contains(&"-i".to_string()) || args.contains(&"--interactive".to_string())
        }
        "add" => {
            // Patch mode
            args.contains(&"-p".to_string()) || args.contains(&"--patch".to_string())
        }
        "checkout" => {
            // Patch mode
            args.contains(&"-p".to_string()) || args.contains(&"--patch".to_string())
        }
        "reset" => {
            // Patch mode
            args.contains(&"-p".to_string()) || args.contains(&"--patch".to_string())
        }
        "difftool" | "mergetool" => true,
        _ => false,
    }
}

fn is_simple_clone(args: &[String]) -> AocGitResult<bool> {
    if !matches!(args.first(), Some(arg) if arg == "clone") {
        return Ok(false);
    }
    if args.len() != 2 {
        return Err(AocGitError::Clone);
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{is_simple_clone, AocGitError};

    #[test]
    fn empty_args_are_not_a_clone() {
        assert!(!is_simple_clone(&[]).expect("empty args are valid"));
    }

    #[test]
    fn clone_requires_exactly_one_repository_argument() {
        assert!(
            is_simple_clone(&["clone".to_string(), "repository".to_string()])
                .expect("simple clone is accepted")
        );
        assert!(matches!(
            is_simple_clone(&["clone".to_string()]),
            Err(AocGitError::Clone)
        ));
    }
}
