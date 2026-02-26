use crate::types::{Command, Rule};
use regex::Regex;

/// Rule that fixes git push when upstream is not set.
///
/// When you try to `git push` without setting upstream, git suggests:
/// `git push --set-upstream origin <branch>`
///
/// This rule extracts that suggestion and uses it.
pub struct GitPushRule;

impl GitPushRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitPushRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for GitPushRule {
    fn name(&self) -> &str {
        "git_push"
    }

    fn matches(&self, command: &Command) -> bool {
        // Must be a git push command
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() || parts[0] != "git" {
            return false;
        }

        if !parts.contains(&"push".to_string()) {
            return false;
        }

        // Check if output suggests setting upstream
        let output = match &command.output {
            Some(out) => out,
            None => return false,
        };

        output.contains("git push --set-upstream")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let output = match &command.output {
            Some(out) => out,
            None => return vec![],
        };

        // Extract the suggested command from git's output
        // Format: "git push --set-upstream origin <branch>"
        let re = Regex::new(r"git push (--set-upstream\s+\S+\s+\S+)").unwrap();

        if let Some(caps) = re.captures(output) {
            let suggestion = caps.get(1).map(|m| m.as_str()).unwrap_or("");

            // Build the new command
            // If the original command had extra flags, we need to handle them
            let mut cmd = command.clone();
            let parts = cmd.script_parts();

            // Find where 'push' is in the command
            let push_idx = parts.iter().position(|p| p == "push").unwrap_or(0);

            // Rebuild with the suggestion
            let mut new_parts: Vec<&str> = parts[..=push_idx].iter().map(|s| s.as_str()).collect();
            new_parts.push(suggestion);

            // Join and return
            let new_command = new_parts.join(" ");
            return vec![new_command];
        }

        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIT_PUSH_OUTPUT: &str = r#"fatal: The current branch feature-branch has no upstream branch.
To push the current branch and set the remote as upstream, use

    git push --set-upstream origin feature-branch
"#;

    #[test]
    fn test_git_push_matches() {
        let rule = GitPushRule::new();
        let cmd = Command::new("git push", Some(GIT_PUSH_OUTPUT.to_string()));
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_git_push_matches_with_remote() {
        let rule = GitPushRule::new();
        let cmd = Command::new("git push origin", Some(GIT_PUSH_OUTPUT.to_string()));
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_git_push_no_match_not_git() {
        let rule = GitPushRule::new();
        let cmd = Command::new("push something", Some(GIT_PUSH_OUTPUT.to_string()));
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_git_push_no_match_different_error() {
        let rule = GitPushRule::new();
        let cmd = Command::new(
            "git push",
            Some("error: failed to push some refs".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_git_push_get_new_command() {
        let rule = GitPushRule::new();
        let cmd = Command::new("git push", Some(GIT_PUSH_OUTPUT.to_string()));
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["git push --set-upstream origin feature-branch"]
        );
    }
}
