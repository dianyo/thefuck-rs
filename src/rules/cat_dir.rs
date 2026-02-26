//! Rule to replace cat with ls when used on a directory.
//!
//! When you try to `cat` a directory, this suggests using `ls` instead.

use crate::types::{Command, Rule};
use std::path::Path;

pub struct CatDirRule;

impl CatDirRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CatDirRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CatDirRule {
    fn name(&self) -> &str {
        "cat_dir"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o,
            None => return false,
        };

        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() || parts[0] != "cat" {
            return false;
        }

        if parts.len() < 2 {
            return false;
        }

        // Check if the output indicates it's a directory
        output.starts_with("cat: ") && Path::new(&parts[1]).is_dir()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let new_script = command.script.replacen("cat", "ls", 1);
        vec![new_script]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_cat_dir_matches() {
        let rule = CatDirRule::new();

        // Use current directory which is guaranteed to exist and be a directory
        let current_dir = env::current_dir().unwrap();
        let dir_str = current_dir.to_string_lossy();

        let cmd = Command::new(
            &format!("cat {}", dir_str),
            Some(format!("cat: {}: Is a directory", dir_str)),
        );

        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cat_dir_no_match_file() {
        let rule = CatDirRule::new();

        let cmd = Command::new(
            "cat Cargo.toml",
            Some("[package]\nname = \"thefuck-rs\"".to_string()),
        );

        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_cat_dir_get_new_command() {
        let rule = CatDirRule::new();

        let cmd = Command::new("cat /tmp", Some("cat: /tmp: Is a directory".to_string()));

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["ls /tmp"]);
    }
}
