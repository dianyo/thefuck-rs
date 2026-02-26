use crate::types::{Command, Rule};
use regex::Regex;

/// Rule that fixes misspelled git commands.
///
/// When you type `git psuh` instead of `git push`, git suggests
/// the correct command in its output.
pub struct GitNotCommandRule;

impl GitNotCommandRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitNotCommandRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for GitNotCommandRule {
    fn name(&self) -> &str {
        "git_not_command"
    }

    fn matches(&self, command: &Command) -> bool {
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.is_empty() || parts[0] != "git" {
            return false;
        }

        let output = match &command.output {
            Some(out) => out,
            None => return false,
        };

        output.contains("is not a git command")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let output = match &command.output {
            Some(out) => out,
            None => return vec![],
        };

        // Git suggests: Did you mean this? / The most similar command is
        let re_most_similar =
            Regex::new(r"(?i)the most similar command(?:s)? (?:is|are)\s*\n\s*(\S+)").unwrap();
        let re_did_you_mean = Regex::new(r"(?i)did you mean this\?\s*\n\s*(\S+)").unwrap();

        let mut suggestions = Vec::new();

        if let Some(caps) = re_most_similar.captures(output) {
            if let Some(m) = caps.get(1) {
                suggestions.push(m.as_str().to_string());
            }
        }

        if let Some(caps) = re_did_you_mean.captures(output) {
            if let Some(m) = caps.get(1) {
                let suggestion = m.as_str().to_string();
                if !suggestions.contains(&suggestion) {
                    suggestions.push(suggestion);
                }
            }
        }

        // Also check for inline suggestions
        let re_inline = Regex::new(r"Did you mean '([^']+)'").unwrap();
        for caps in re_inline.captures_iter(output) {
            if let Some(m) = caps.get(1) {
                let suggestion = m.as_str().to_string();
                if !suggestions.contains(&suggestion) {
                    suggestions.push(suggestion);
                }
            }
        }

        // Build the corrected commands
        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.len() >= 2 {
            let misspelled = &parts[1];
            suggestions
                .into_iter()
                .map(|correct| command.script.replacen(misspelled, &correct, 1))
                .collect()
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIT_NOT_COMMAND_OUTPUT: &str = r#"git: 'psuh' is not a git command. See 'git --help'.

The most similar command is
	push
"#;

    #[test]
    fn test_git_not_command_matches() {
        let rule = GitNotCommandRule::new();
        let cmd = Command::new(
            "git psuh origin main",
            Some(GIT_NOT_COMMAND_OUTPUT.to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_git_not_command_no_match_not_git() {
        let rule = GitNotCommandRule::new();
        let cmd = Command::new("psuh origin main", Some(GIT_NOT_COMMAND_OUTPUT.to_string()));
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_git_not_command_no_match_valid_command() {
        let rule = GitNotCommandRule::new();
        let cmd = Command::new(
            "git push origin main",
            Some("Everything up-to-date".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_git_not_command_get_new_command() {
        let rule = GitNotCommandRule::new();
        let cmd = Command::new(
            "git psuh origin main",
            Some(GIT_NOT_COMMAND_OUTPUT.to_string()),
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["git push origin main"]);
    }
}
