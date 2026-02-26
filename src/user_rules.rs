//! User-defined rules support.
//!
//! Allows users to create custom rules using TOML configuration files.
//! Rules are loaded from ~/.config/thefuck-rs/rules/

use crate::config::Settings;
use crate::types::{Command, Rule, DEFAULT_PRIORITY};
use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

/// A user-defined rule loaded from a TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct UserRule {
    /// Rule name
    pub name: String,

    /// Whether the rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rule priority
    #[serde(default = "default_priority")]
    pub priority: i32,

    /// Pattern to match against the command script
    #[serde(default)]
    pub match_script: Option<String>,

    /// Pattern to match against the command output
    #[serde(default)]
    pub match_output: Option<String>,

    /// Fixed replacement command
    #[serde(default)]
    pub new_command: Option<String>,

    /// Replacement pattern (with capture groups)
    #[serde(default)]
    pub new_command_pattern: Option<String>,

    /// Whether the rule requires output to match
    #[serde(default = "default_true")]
    pub requires_output: bool,

    // Compiled regexes (not deserialized)
    #[serde(skip)]
    script_regex: Option<Regex>,
    #[serde(skip)]
    output_regex: Option<Regex>,
}

fn default_true() -> bool {
    true
}

fn default_priority() -> i32 {
    DEFAULT_PRIORITY
}

impl UserRule {
    /// Compiles the regex patterns.
    pub fn compile_patterns(&mut self) -> Result<(), regex::Error> {
        if let Some(ref pattern) = self.match_script {
            self.script_regex = Some(Regex::new(pattern)?);
        }
        if let Some(ref pattern) = self.match_output {
            self.output_regex = Some(Regex::new(pattern)?);
        }
        Ok(())
    }
}

impl Rule for UserRule {
    fn name(&self) -> &str {
        &self.name
    }

    fn matches(&self, command: &Command) -> bool {
        // Check script pattern
        if let Some(ref regex) = self.script_regex {
            if !regex.is_match(&command.script) {
                return false;
            }
        }

        // Check output pattern
        if let Some(ref regex) = self.output_regex {
            if let Some(ref output) = command.output {
                if !regex.is_match(output) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // At least one pattern must be defined
        self.script_regex.is_some() || self.output_regex.is_some()
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Fixed replacement
        if let Some(ref new_cmd) = self.new_command {
            return vec![new_cmd.clone()];
        }

        // Pattern-based replacement
        if let Some(ref pattern) = self.new_command_pattern {
            if let Some(ref regex) = self.script_regex {
                let result = regex.replace(&command.script, pattern.as_str());
                return vec![result.to_string()];
            }
        }

        vec![]
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    fn enabled_by_default(&self) -> bool {
        self.enabled
    }

    fn requires_output(&self) -> bool {
        self.requires_output
    }
}

/// Loads all user-defined rules from the rules directory.
pub fn load_user_rules() -> Vec<Box<dyn Rule>> {
    let rules_dir = match Settings::user_rules_dir() {
        Some(d) => d,
        None => return vec![],
    };

    if !rules_dir.exists() {
        return vec![];
    }

    let mut rules: Vec<Box<dyn Rule>> = Vec::new();

    let entries = match fs::read_dir(&rules_dir) {
        Ok(e) => e,
        Err(_) => return vec![],
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "toml") {
            if let Some(rule) = load_rule_from_file(&path) {
                rules.push(Box::new(rule));
            }
        }
    }

    tracing::debug!("Loaded {} user rules from {:?}", rules.len(), rules_dir);
    rules
}

/// Loads a single rule from a TOML file.
fn load_rule_from_file(path: &PathBuf) -> Option<UserRule> {
    let content = fs::read_to_string(path).ok()?;

    let mut rule: UserRule = match toml::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to parse rule file {:?}: {}", path, e);
            return None;
        }
    };

    if let Err(e) = rule.compile_patterns() {
        tracing::warn!("Failed to compile patterns in {:?}: {}", path, e);
        return None;
    }

    Some(rule)
}

/// Creates the user rules directory if it doesn't exist.
pub fn init_user_rules_dir() -> std::io::Result<PathBuf> {
    let rules_dir = Settings::user_rules_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine rules directory",
        )
    })?;

    if !rules_dir.exists() {
        fs::create_dir_all(&rules_dir)?;

        // Create an example rule file
        let example_path = rules_dir.join("example.toml.disabled");
        let example_content = r#"# Example user-defined rule
# Remove the .disabled extension to enable this rule

name = "my_custom_rule"
enabled = true
priority = 1000

# Match pattern for the command script (regex)
match_script = "^mycommand (.+)$"

# Match pattern for the command output (regex, optional)
# match_output = "error: (.+)"

# Fixed replacement command
# new_command = "mycommand --fixed"

# Or use pattern-based replacement with capture groups
new_command_pattern = "mycommand --correct $1"

# Whether this rule requires command output to match
requires_output = false
"#;
        fs::write(&example_path, example_content)?;
    }

    Ok(rules_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_rule_matches() {
        let mut rule = UserRule {
            name: "test".to_string(),
            enabled: true,
            priority: 1000,
            match_script: Some("^git psuh".to_string()),
            match_output: None,
            new_command: Some("git push".to_string()),
            new_command_pattern: None,
            requires_output: false,
            script_regex: None,
            output_regex: None,
        };
        rule.compile_patterns().unwrap();

        let cmd = Command::new("git psuh origin main", None);
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_user_rule_get_new_command() {
        let mut rule = UserRule {
            name: "test".to_string(),
            enabled: true,
            priority: 1000,
            match_script: Some("^git psuh (.+)$".to_string()),
            match_output: None,
            new_command: None,
            new_command_pattern: Some("git push $1".to_string()),
            requires_output: false,
            script_regex: None,
            output_regex: None,
        };
        rule.compile_patterns().unwrap();

        let cmd = Command::new("git psuh origin main", None);
        let result = rule.get_new_command(&cmd);
        assert_eq!(result, vec!["git push origin main"]);
    }

    #[test]
    fn test_user_rule_with_output() {
        let mut rule = UserRule {
            name: "test".to_string(),
            enabled: true,
            priority: 1000,
            match_script: Some("^cat ".to_string()),
            match_output: Some("Is a directory".to_string()),
            new_command: Some("ls".to_string()),
            new_command_pattern: None,
            requires_output: true,
            script_regex: None,
            output_regex: None,
        };
        rule.compile_patterns().unwrap();

        let cmd_with_output =
            Command::new("cat /tmp", Some("cat: /tmp: Is a directory".to_string()));
        assert!(rule.matches(&cmd_with_output));

        let cmd_without_output = Command::new("cat /tmp", None);
        assert!(!rule.matches(&cmd_without_output));
    }
}
