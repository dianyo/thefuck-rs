//! Rule to add missing files before git operations.
//!
//! When a git command fails because a file isn't tracked, this rule
//! suggests adding the file first.

use crate::types::{Command, Rule};
use regex::Regex;

pub struct GitAddRule {
    pathspec_re: Regex,
}

impl GitAddRule {
    pub fn new() -> Self {
        Self {
            pathspec_re: Regex::new(
                r"error: pathspec '([^']*)' did not match any file\(s\) known to git",
            )
            .unwrap(),
        }
    }

    fn get_missing_file(&self, command: &Command) -> Option<String> {
        let output = command.output.as_ref()?;
        self.pathspec_re
            .captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Default for GitAddRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for GitAddRule {
    fn name(&self) -> &str {
        "git_add"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o,
            None => return false,
        };

        command.script.starts_with("git ")
            && output.contains("did not match any file(s) known to git")
            && output.contains("Did you forget to 'git add'?")
            && self.get_missing_file(command).is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        if let Some(missing_file) = self.get_missing_file(command) {
            vec![format!("git add -- {} && {}", missing_file, command.script)]
        } else {
            vec![]
        }
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_add_matches() {
        let rule = GitAddRule::new();

        let cmd = Command::new(
            "git commit -m 'test'",
            Some("error: pathspec 'newfile.txt' did not match any file(s) known to git\nDid you forget to 'git add'?".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_git_add_no_match_without_hint() {
        let rule = GitAddRule::new();

        let cmd = Command::new(
            "git checkout feature",
            Some("error: pathspec 'feature' did not match any file(s) known to git".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_git_add_get_new_command() {
        let rule = GitAddRule::new();

        let cmd = Command::new(
            "git commit -m 'test'",
            Some("error: pathspec 'newfile.txt' did not match any file(s) known to git\nDid you forget to 'git add'?".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(
            result,
            vec!["git add -- newfile.txt && git commit -m 'test'"]
        );
    }
}
