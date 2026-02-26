use crate::types::{Command, Rule};
use regex::Regex;

/// Rule that adds -p flag to mkdir when parent directories don't exist.
pub struct MkdirPRule;

impl MkdirPRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MkdirPRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for MkdirPRule {
    fn name(&self) -> &str {
        "mkdir_p"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(out) => out,
            None => return false,
        };

        command.script.contains("mkdir") && output.contains("No such file or directory")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Replace `mkdir` with `mkdir -p`, handling various forms:
        // - mkdir dir -> mkdir -p dir
        // - mkdir -m 755 dir -> mkdir -p -m 755 dir
        let re = Regex::new(r"\bmkdir\s+").unwrap();
        let result = re.replace(&command.script, "mkdir -p ").to_string();
        vec![result]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkdir_p_matches() {
        let rule = MkdirPRule::new();
        let cmd = Command::new(
            "mkdir /path/to/new/dir",
            Some("mkdir: cannot create directory '/path/to/new/dir': No such file or directory".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_mkdir_p_no_match_success() {
        let rule = MkdirPRule::new();
        let cmd = Command::new("mkdir mydir", Some("".to_string()));
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_mkdir_p_no_match_wrong_command() {
        let rule = MkdirPRule::new();
        let cmd = Command::new("touch file", Some("No such file or directory".to_string()));
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_mkdir_p_get_new_command() {
        let rule = MkdirPRule::new();
        let cmd = Command::new("mkdir /path/to/dir", Some("No such file or directory".to_string()));
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["mkdir -p /path/to/dir"]);
    }

    #[test]
    fn test_mkdir_p_get_new_command_with_flags() {
        let rule = MkdirPRule::new();
        let cmd = Command::new("mkdir -m 755 /path/to/dir", Some("No such file or directory".to_string()));
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["mkdir -p -m 755 /path/to/dir"]);
    }
}
