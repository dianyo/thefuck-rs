//! Rule to fix missing space after `man`.
//!
//! When you type `mangit` instead of `man git`, this rule adds the space.

use crate::types::{Command, Rule, DEFAULT_PRIORITY};

pub struct ManNoSpaceRule;

impl ManNoSpaceRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ManNoSpaceRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for ManNoSpaceRule {
    fn name(&self) -> &str {
        "man_no_space"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o.to_lowercase(),
            None => return false,
        };

        command.script.starts_with("man")
            && !command.script.starts_with("man ")
            && output.contains("command not found")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let rest = &command.script[3..];
        vec![format!("man {}", rest)]
    }

    fn priority(&self) -> i32 {
        DEFAULT_PRIORITY + 1000
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_man_no_space_matches() {
        let rule = ManNoSpaceRule::new();

        let cmd = Command::new(
            "mangit",
            Some("bash: mangit: command not found".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_man_no_space_no_match_with_space() {
        let rule = ManNoSpaceRule::new();

        let cmd = Command::new(
            "man git",
            Some("No manual entry for git".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_man_no_space_get_new_command() {
        let rule = ManNoSpaceRule::new();

        let cmd = Command::new(
            "mangit",
            Some("command not found".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["man git"]);
    }

    #[test]
    fn test_man_no_space_compound() {
        let rule = ManNoSpaceRule::new();

        let cmd = Command::new(
            "mangit-status",
            Some("command not found".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["man git-status"]);
    }
}
