//! Rule to fix open command on different platforms.
//!
//! On Linux, `open` might not work; suggests `xdg-open` instead.
//! On macOS, if file doesn't exist, suggests alternatives.

use crate::types::{Command, Rule};

pub struct OpenRule;

impl OpenRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OpenRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for OpenRule {
    fn name(&self) -> &str {
        "open"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o.to_lowercase(),
            None => return false,
        };

        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() {
            return false;
        }

        let first = &parts[0];

        // Linux: open command not found
        (first == "open" && output.contains("command not found"))
            // Or xdg-open with file not found
            || (first == "xdg-open" && output.contains("no such file"))
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() {
            return vec![];
        }

        if parts[0] == "open" {
            // On Linux, suggest xdg-open
            return vec![command.script.replacen("open", "xdg-open", 1)];
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
    fn test_open_matches_not_found() {
        let rule = OpenRule::new();

        let cmd = Command::new(
            "open file.pdf",
            Some("bash: open: command not found".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_open_no_match_success() {
        let rule = OpenRule::new();

        let cmd = Command::new("open .", Some("".to_string()));
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_open_get_new_command() {
        let rule = OpenRule::new();

        let cmd = Command::new("open document.pdf", Some("command not found".to_string()));

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["xdg-open document.pdf"]);
    }
}
