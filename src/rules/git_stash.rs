//! Rule to fix git stash operations.
//!
//! When you try to pop/apply a stash that doesn't exist,
//! this suggests listing stashes first.

use crate::types::{Command, Rule};

pub struct GitStashRule;

impl GitStashRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitStashRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for GitStashRule {
    fn name(&self) -> &str {
        "git_stash"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o,
            None => return false,
        };

        command.script.starts_with("git stash")
            && (output.contains("No stash entries found")
                || output.contains("does not apply to a stash-like commit")
                || output.contains("is not a valid reference"))
    }

    fn get_new_command(&self, _command: &Command) -> Vec<String> {
        vec!["git stash list".to_string()]
    }

    fn requires_output(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_stash_matches_no_entries() {
        let rule = GitStashRule::new();

        let cmd = Command::new(
            "git stash pop",
            Some("No stash entries found.".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_git_stash_no_match_success() {
        let rule = GitStashRule::new();

        let cmd = Command::new(
            "git stash pop",
            Some("On branch main\nChanges not staged for commit:".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_git_stash_get_new_command() {
        let rule = GitStashRule::new();

        let cmd = Command::new(
            "git stash pop",
            Some("No stash entries found.".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["git stash list"]);
    }
}
