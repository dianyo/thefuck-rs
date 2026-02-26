//! Rule to create parent directories for touch.
//!
//! When touch fails because parent directory doesn't exist,
//! this suggests creating the directory first.

use crate::types::{Command, Rule};
use std::path::Path;

pub struct TouchRule;

impl TouchRule {
    pub fn new() -> Self {
        Self
    }

    fn get_directory_to_create(command: &Command) -> Option<String> {
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.len() < 2 {
            return None;
        }

        let file_path = &parts[1];
        let path = Path::new(file_path);

        path.parent()
            .filter(|p| !p.as_os_str().is_empty() && !p.exists())
            .map(|p| p.to_string_lossy().to_string())
    }
}

impl Default for TouchRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for TouchRule {
    fn name(&self) -> &str {
        "touch"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o.to_lowercase(),
            None => return false,
        };

        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() || parts[0] != "touch" {
            return false;
        }

        (output.contains("no such file or directory")
            || output.contains("cannot touch")
            || output.contains("not a directory"))
            && Self::get_directory_to_create(command).is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        if let Some(dir) = Self::get_directory_to_create(command) {
            return vec![format!("mkdir -p {} && {}", dir, command.script)];
        }
        vec![]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_matches() {
        let rule = TouchRule::new();

        let cmd = Command::new(
            "touch /nonexistent/path/file.txt",
            Some("touch: cannot touch '/nonexistent/path/file.txt': No such file or directory".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_touch_no_match_success() {
        let rule = TouchRule::new();

        let cmd = Command::new(
            "touch file.txt",
            Some("".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_touch_get_new_command() {
        let rule = TouchRule::new();

        let cmd = Command::new(
            "touch /nonexistent/path/file.txt",
            Some("No such file or directory".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["mkdir -p /nonexistent/path && touch /nonexistent/path/file.txt"]);
    }
}
