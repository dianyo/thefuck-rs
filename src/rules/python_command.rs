//! Rule to prepend `python` to .py scripts.
//!
//! When you try to run a .py file directly and get a permission error
//! or command not found, this suggests prepending `python`.

use crate::types::{Command, Rule};

pub struct PythonCommandRule;

impl PythonCommandRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PythonCommandRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for PythonCommandRule {
    fn name(&self) -> &str {
        "python_command"
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

        let script = &parts[0];

        script.ends_with(".py")
            && (output.contains("permission denied") || output.contains("command not found"))
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        vec![format!("python {}", command.script)]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_command_matches_permission_denied() {
        let rule = PythonCommandRule::new();

        let cmd = Command::new(
            "script.py",
            Some("bash: script.py: Permission denied".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_python_command_matches_not_found() {
        let rule = PythonCommandRule::new();

        let cmd = Command::new(
            "test.py arg1",
            Some("bash: test.py: command not found".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_python_command_no_match_not_python() {
        let rule = PythonCommandRule::new();

        let cmd = Command::new(
            "script.sh",
            Some("Permission denied".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_python_command_get_new_command() {
        let rule = PythonCommandRule::new();

        let cmd = Command::new(
            "script.py --flag value",
            Some("Permission denied".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["python script.py --flag value"]);
    }
}
