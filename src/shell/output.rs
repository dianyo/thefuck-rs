use crate::config::Settings;
use crate::error::{Result, TheFuckError};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Gets the output of a command by re-running it.
///
/// This is the default method - we re-execute the command and capture its output.
/// The command is run with a timeout based on whether it's a slow command.
pub fn get_output(script: &str, expanded: &str, settings: &Settings) -> Result<Option<String>> {
    let timeout = settings.get_timeout(script);

    tracing::debug!(
        "Re-running command: {} (timeout: {}s)",
        expanded,
        timeout
    );

    // Build environment
    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.extend(settings.env.clone());

    // Run the command
    let output = run_with_timeout(expanded, &env, Duration::from_secs(timeout))?;

    match output {
        Some(out) => {
            tracing::debug!("Received output ({} bytes)", out.len());
            Ok(Some(out))
        }
        None => {
            tracing::debug!("Command timed out");
            Ok(None)
        }
    }
}

/// Runs a command with a timeout.
fn run_with_timeout(
    command: &str,
    env: &HashMap<String, String>,
    timeout: Duration,
) -> Result<Option<String>> {
    // Determine the shell to use
    let shell = if cfg!(windows) { "cmd" } else { "sh" };
    let shell_arg = if cfg!(windows) { "/C" } else { "-c" };

    let mut child = Command::new(shell)
        .arg(shell_arg)
        .arg(command)
        .envs(env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| TheFuckError::ExecutionError(e.to_string()))?;

    // Wait for the command with timeout
    match wait_with_timeout(&mut child, timeout) {
        Ok(true) => {
            // Command completed - get output
            let output = child
                .wait_with_output()
                .map_err(|e| TheFuckError::ExecutionError(e.to_string()))?;

            // Combine stdout and stderr
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let combined = if !stderr.is_empty() {
                format!("{}{}", stdout, stderr)
            } else {
                stdout.to_string()
            };

            Ok(Some(combined))
        }
        Ok(false) => {
            // Timeout - kill the process
            let _ = child.kill();
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

/// Waits for a child process with a timeout.
/// Returns true if the process completed, false if it timed out.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<bool> {
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(100);

    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(true), // Process completed
            Ok(None) => {
                // Still running
                if start.elapsed() >= timeout {
                    return Ok(false); // Timed out
                }
                std::thread::sleep(poll_interval);
            }
            Err(e) => return Err(TheFuckError::ExecutionError(e.to_string())),
        }
    }
}

/// Extracts the raw command from TF_HISTORY environment variable.
///
/// The shell alias sets TF_HISTORY with the last N commands from history.
/// We need to extract the most recent failed command.
pub fn get_raw_command_from_history(history: &str) -> Option<String> {
    // TF_HISTORY contains multiple lines from fc -ln -10
    // The last non-empty line is typically the failed command
    // But we need to skip the 'thefuck' or alias invocation itself

    let lines: Vec<&str> = history
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    // Find the last command that isn't the thefuck invocation
    for line in lines.iter().rev() {
        let lower = line.to_lowercase();
        // Skip if it's the fuck/thefuck command itself
        if !lower.starts_with("fuck")
            && !lower.starts_with("thefuck")
            && !lower.contains("tf_alias")
        {
            return Some(line.to_string());
        }
    }

    // If all else fails, return the second-to-last line
    if lines.len() >= 2 {
        Some(lines[lines.len() - 2].to_string())
    } else {
        lines.first().map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_raw_command_from_history() {
        let history = "git status\ngit commit -m 'test'\ngit psuh origin main\nfuck";
        let result = get_raw_command_from_history(history);
        assert_eq!(result, Some("git psuh origin main".to_string()));
    }

    #[test]
    fn test_get_raw_command_from_history_single() {
        let history = "git psuh origin main";
        let result = get_raw_command_from_history(history);
        assert_eq!(result, Some("git psuh origin main".to_string()));
    }

    #[test]
    fn test_get_raw_command_skips_fuck() {
        let history = "bad_command\nfuck";
        let result = get_raw_command_from_history(history);
        assert_eq!(result, Some("bad_command".to_string()));
    }
}
