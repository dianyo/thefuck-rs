//! Rule to fix cargo subcommand typos.
//!
//! When cargo reports "no such command" with a suggestion,
//! this rule applies the suggested fix.

use crate::types::{Command, Rule};
use regex::Regex;

pub struct CargoNoCommandRule {
    suggestion_re: Regex,
    suggestion_re_alt: Regex,
}

impl CargoNoCommandRule {
    pub fn new() -> Self {
        Self {
            // Old format: Did you mean `build`?
            suggestion_re: Regex::new(r"Did you mean `([^`]*)`").unwrap(),
            // New format: a command with a similar name exists: `build`
            suggestion_re_alt: Regex::new(r"a command with a similar name exists: `([^`]*)`")
                .unwrap(),
        }
    }

    fn get_suggestion(&self, output: &str) -> Option<String> {
        // Try new format first
        if let Some(caps) = self.suggestion_re_alt.captures(output) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }

        // Fall back to old format
        self.suggestion_re
            .captures(output)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}

impl Default for CargoNoCommandRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CargoNoCommandRule {
    fn name(&self) -> &str {
        "cargo_no_command"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(o) => o.to_lowercase(),
            None => return false,
        };

        let original_output = command.output.as_ref().unwrap();

        command.script.starts_with("cargo ")
            && (output.contains("no such subcommand") || output.contains("no such command"))
            && (original_output.contains("Did you mean")
                || original_output.contains("a command with a similar name exists"))
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let output = match &command.output {
            Some(o) => o,
            None => return vec![],
        };

        let mut cmd = command.clone();
        let parts = cmd.script_parts();

        if parts.len() < 2 {
            return vec![];
        }

        let broken = &parts[1];

        if let Some(fix) = self.get_suggestion(output) {
            let new_script = command.script.replacen(broken, &fix, 1);
            return vec![new_script];
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
    fn test_cargo_no_command_matches_old_format() {
        let rule = CargoNoCommandRule::new();

        let cmd = Command::new(
            "cargo buld",
            Some("error: no such subcommand: `buld`\n\n\tDid you mean `build`?".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cargo_no_command_matches_new_format() {
        let rule = CargoNoCommandRule::new();

        let cmd = Command::new(
            "cargo buld",
            Some(
                "error: no such command: `buld`\n\nhelp: a command with a similar name exists: `build`"
                    .to_string(),
            ),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cargo_no_command_no_match_valid() {
        let rule = CargoNoCommandRule::new();

        let cmd = Command::new("cargo build", Some("Compiling...".to_string()));
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_cargo_no_command_get_new_command_old_format() {
        let rule = CargoNoCommandRule::new();

        let cmd = Command::new(
            "cargo buld --release",
            Some("error: no such subcommand: `buld`\n\n\tDid you mean `build`?".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["cargo build --release"]);
    }

    #[test]
    fn test_cargo_no_command_get_new_command_new_format() {
        let rule = CargoNoCommandRule::new();

        let cmd = Command::new(
            "cargo buld --release",
            Some(
                "error: no such command: `buld`\n\nhelp: a command with a similar name exists: `build`"
                    .to_string(),
            ),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["cargo build --release"]);
    }

    #[test]
    fn test_cargo_no_command_test_typo() {
        let rule = CargoNoCommandRule::new();

        let cmd = Command::new(
            "cargo tset",
            Some("error: no such subcommand: `tset`\n\n\tDid you mean `test`?".to_string()),
        );

        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["cargo test"]);
    }
}
