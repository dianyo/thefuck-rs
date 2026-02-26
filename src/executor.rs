//! Command execution module.
//!
//! Handles running corrected commands after selection.

use std::io;
use std::process::{Command, ExitStatus, Stdio};

/// Executes a command and returns its exit status.
pub fn execute_command(script: &str) -> io::Result<ExitStatus> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    Command::new(&shell)
        .arg("-c")
        .arg(script)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

/// Executes a command and captures its output.
pub fn execute_command_capture(script: &str) -> io::Result<(ExitStatus, String, String)> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let output = Command::new(&shell).arg("-c").arg(script).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((output.status, stdout, stderr))
}

/// Checks if a command exists in PATH.
pub fn command_exists(cmd: &str) -> bool {
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| {
            let path = std::path::Path::new(dir).join(cmd);
            path.exists() && path.is_file()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_capture() {
        let (status, stdout, _stderr) = execute_command_capture("echo hello").unwrap();
        assert!(status.success());
        assert_eq!(stdout.trim(), "hello");
    }

    #[test]
    fn test_command_exists() {
        assert!(command_exists("ls"));
        assert!(command_exists("echo"));
        assert!(!command_exists("nonexistent_command_12345"));
    }
}
