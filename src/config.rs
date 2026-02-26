use crate::error::{Result, TheFuckError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Special constant indicating all rules are enabled.
pub const ALL_ENABLED: &str = "ALL";

/// Default priority for rules.
pub const DEFAULT_PRIORITY: i32 = 1000;

/// Application settings.
///
/// Settings can be loaded from:
/// 1. Default values
/// 2. Config file (~/.config/thefuck/settings.toml)
/// 3. Environment variables (THEFUCK_*)
/// 4. Command-line arguments
///
/// Later sources override earlier ones.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// List of enabled rules. Use ["ALL"] to enable all rules.
    pub rules: Vec<String>,

    /// List of rules to exclude.
    pub exclude_rules: Vec<String>,

    /// Seconds to wait for command output.
    pub wait_command: u64,

    /// Seconds to wait for slow commands.
    pub wait_slow_command: u64,

    /// Whether to require confirmation before running fixed command.
    pub require_confirmation: bool,

    /// Disable colored output.
    pub no_colors: bool,

    /// Enable debug output.
    pub debug: bool,

    /// Per-rule priority overrides.
    pub priority: HashMap<String, i32>,

    /// Maximum number of history entries to scan.
    pub history_limit: Option<usize>,

    /// Add fixed command to shell history.
    pub alter_history: bool,

    /// Commands that are known to be slow.
    pub slow_commands: Vec<String>,

    /// Repeat thefuck if fixed command also fails.
    pub repeat: bool,

    /// Use instant mode (read output from log instead of re-running).
    pub instant_mode: bool,

    /// Number of close matches to suggest.
    pub num_close_matches: usize,

    /// Environment variables to set when running commands.
    pub env: HashMap<String, String>,

    /// Path prefixes to exclude when searching for executables.
    pub excluded_search_path_prefixes: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("LC_ALL".to_string(), "C".to_string());
        env_vars.insert("LANG".to_string(), "C".to_string());
        env_vars.insert("GIT_TRACE".to_string(), "1".to_string());

        Self {
            rules: vec![ALL_ENABLED.to_string()],
            exclude_rules: vec![],
            wait_command: 3,
            wait_slow_command: 15,
            require_confirmation: true,
            no_colors: false,
            debug: false,
            priority: HashMap::new(),
            history_limit: None,
            alter_history: true,
            slow_commands: vec![
                "lein".to_string(),
                "react-native".to_string(),
                "gradle".to_string(),
                "./gradlew".to_string(),
                "vagrant".to_string(),
            ],
            repeat: false,
            instant_mode: false,
            num_close_matches: 3,
            env: env_vars,
            excluded_search_path_prefixes: vec![],
        }
    }
}

impl Settings {
    /// Creates settings from defaults, config file, env vars, and CLI args.
    pub fn load() -> Result<Self> {
        let mut settings = Self::default();

        // Load from config file
        if let Some(config_path) = Self::config_file_path() {
            if config_path.exists() {
                settings.merge_from_file(&config_path)?;
            }
        }

        // Load from environment variables
        settings.merge_from_env();

        Ok(settings)
    }

    /// Returns the config directory path.
    pub fn config_dir() -> Option<PathBuf> {
        // Check XDG_CONFIG_HOME first
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg_config).join("thefuck");
            return Some(path);
        }

        // Check for legacy ~/.thefuck directory
        if let Some(home) = dirs::home_dir() {
            let legacy_path = home.join(".thefuck");
            if legacy_path.is_dir() {
                tracing::warn!(
                    "Using legacy config path {:?}. Consider moving to ~/.config/thefuck/",
                    legacy_path
                );
                return Some(legacy_path);
            }
        }

        // Use standard config directory
        dirs::config_dir().map(|p| p.join("thefuck"))
    }

    /// Returns the config file path.
    pub fn config_file_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("settings.toml"))
    }

    /// Returns the user rules directory path.
    pub fn user_rules_dir() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("rules"))
    }

    /// Returns the cache directory path.
    pub fn cache_dir() -> Option<PathBuf> {
        dirs::cache_dir().map(|p| p.join("thefuck"))
    }

    /// Ensures the config directory and files exist.
    pub fn init_config_dir() -> Result<PathBuf> {
        let config_dir = Self::config_dir()
            .ok_or_else(|| TheFuckError::ConfigError("Cannot determine config directory".into()))?;

        // Create config directory
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        // Create rules subdirectory
        let rules_dir = config_dir.join("rules");
        if !rules_dir.exists() {
            fs::create_dir_all(&rules_dir)?;
        }

        // Create default config file if it doesn't exist
        let config_file = config_dir.join("settings.toml");
        if !config_file.exists() {
            let default_config = Self::default_config_content();
            fs::write(&config_file, default_config)?;
        }

        Ok(config_dir)
    }

    /// Returns the default config file content.
    fn default_config_content() -> String {
        r#"# thefuck-rs settings
# See https://github.com/nvbn/thefuck#settings for more information.

# Enabled rules. Use ["ALL"] to enable all rules.
# rules = ["ALL"]

# Rules to exclude
# exclude_rules = []

# Seconds to wait for command output
# wait_command = 3

# Seconds to wait for slow commands
# wait_slow_command = 15

# Require confirmation before running fixed command
# require_confirmation = true

# Disable colored output
# no_colors = false

# Enable debug output
# debug = false

# Maximum history entries to scan (null for unlimited)
# history_limit = null

# Add fixed command to shell history
# alter_history = true

# Commands known to be slow
# slow_commands = ["lein", "react-native", "gradle", "./gradlew", "vagrant"]

# Repeat thefuck if fixed command also fails
# repeat = false

# Number of close matches to suggest
# num_close_matches = 3
"#
        .to_string()
    }

    /// Merges settings from a TOML config file.
    fn merge_from_file(&mut self, path: &PathBuf) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let file_settings: SettingsPartial = toml::from_str(&content)?;

        // Merge each field if present
        if let Some(rules) = file_settings.rules {
            self.rules = rules;
        }
        if let Some(exclude_rules) = file_settings.exclude_rules {
            self.exclude_rules = exclude_rules;
        }
        if let Some(wait_command) = file_settings.wait_command {
            self.wait_command = wait_command;
        }
        if let Some(wait_slow_command) = file_settings.wait_slow_command {
            self.wait_slow_command = wait_slow_command;
        }
        if let Some(require_confirmation) = file_settings.require_confirmation {
            self.require_confirmation = require_confirmation;
        }
        if let Some(no_colors) = file_settings.no_colors {
            self.no_colors = no_colors;
        }
        if let Some(debug) = file_settings.debug {
            self.debug = debug;
        }
        if let Some(priority) = file_settings.priority {
            self.priority = priority;
        }
        if let Some(history_limit) = file_settings.history_limit {
            self.history_limit = history_limit;
        }
        if let Some(alter_history) = file_settings.alter_history {
            self.alter_history = alter_history;
        }
        if let Some(slow_commands) = file_settings.slow_commands {
            self.slow_commands = slow_commands;
        }
        if let Some(repeat) = file_settings.repeat {
            self.repeat = repeat;
        }
        if let Some(instant_mode) = file_settings.instant_mode {
            self.instant_mode = instant_mode;
        }
        if let Some(num_close_matches) = file_settings.num_close_matches {
            self.num_close_matches = num_close_matches;
        }
        if let Some(env) = file_settings.env {
            self.env.extend(env);
        }
        if let Some(excluded_search_path_prefixes) = file_settings.excluded_search_path_prefixes {
            self.excluded_search_path_prefixes = excluded_search_path_prefixes;
        }

        Ok(())
    }

    /// Merges settings from environment variables.
    fn merge_from_env(&mut self) {
        // THEFUCK_RULES - colon-separated list
        if let Ok(val) = env::var("THEFUCK_RULES") {
            self.rules = Self::parse_rules_env(&val);
        }

        // THEFUCK_EXCLUDE_RULES - colon-separated list
        if let Ok(val) = env::var("THEFUCK_EXCLUDE_RULES") {
            self.exclude_rules = val.split(':').map(String::from).collect();
        }

        // THEFUCK_WAIT_COMMAND
        if let Ok(val) = env::var("THEFUCK_WAIT_COMMAND") {
            if let Ok(n) = val.parse() {
                self.wait_command = n;
            }
        }

        // THEFUCK_WAIT_SLOW_COMMAND
        if let Ok(val) = env::var("THEFUCK_WAIT_SLOW_COMMAND") {
            if let Ok(n) = val.parse() {
                self.wait_slow_command = n;
            }
        }

        // THEFUCK_REQUIRE_CONFIRMATION
        if let Ok(val) = env::var("THEFUCK_REQUIRE_CONFIRMATION") {
            self.require_confirmation = val.eq_ignore_ascii_case("true");
        }

        // THEFUCK_NO_COLORS
        if let Ok(val) = env::var("THEFUCK_NO_COLORS") {
            self.no_colors = val.eq_ignore_ascii_case("true");
        }

        // THEFUCK_DEBUG
        if let Ok(val) = env::var("THEFUCK_DEBUG") {
            self.debug = val.eq_ignore_ascii_case("true");
        }

        // THEFUCK_PRIORITY - colon-separated rule=priority pairs
        if let Ok(val) = env::var("THEFUCK_PRIORITY") {
            self.priority = Self::parse_priority_env(&val);
        }

        // THEFUCK_HISTORY_LIMIT
        if let Ok(val) = env::var("THEFUCK_HISTORY_LIMIT") {
            if let Ok(n) = val.parse() {
                self.history_limit = Some(n);
            }
        }

        // THEFUCK_ALTER_HISTORY
        if let Ok(val) = env::var("THEFUCK_ALTER_HISTORY") {
            self.alter_history = val.eq_ignore_ascii_case("true");
        }

        // THEFUCK_SLOW_COMMANDS - colon-separated list
        if let Ok(val) = env::var("THEFUCK_SLOW_COMMANDS") {
            self.slow_commands = val.split(':').map(String::from).collect();
        }

        // THEFUCK_REPEAT
        if let Ok(val) = env::var("THEFUCK_REPEAT") {
            self.repeat = val.eq_ignore_ascii_case("true");
        }

        // THEFUCK_INSTANT_MODE
        if let Ok(val) = env::var("THEFUCK_INSTANT_MODE") {
            self.instant_mode = val.eq_ignore_ascii_case("true");
        }

        // THEFUCK_NUM_CLOSE_MATCHES
        if let Ok(val) = env::var("THEFUCK_NUM_CLOSE_MATCHES") {
            if let Ok(n) = val.parse() {
                self.num_close_matches = n;
            }
        }

        // THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES - colon-separated list
        if let Ok(val) = env::var("THEFUCK_EXCLUDED_SEARCH_PATH_PREFIXES") {
            self.excluded_search_path_prefixes = val.split(':').map(String::from).collect();
        }
    }

    /// Merges settings from CLI arguments.
    pub fn merge_from_args(&mut self, debug: bool, repeat: bool, yes: bool) {
        if debug {
            self.debug = true;
        }
        if repeat {
            self.repeat = true;
        }
        if yes {
            self.require_confirmation = false;
        }
    }

    /// Parses rules from environment variable format (colon-separated).
    fn parse_rules_env(val: &str) -> Vec<String> {
        let parts: Vec<&str> = val.split(':').collect();
        let mut rules = Vec::new();

        for part in parts {
            if part == "DEFAULT_RULES" {
                rules.push(ALL_ENABLED.to_string());
            } else if !part.is_empty() {
                rules.push(part.to_string());
            }
        }

        if rules.is_empty() {
            rules.push(ALL_ENABLED.to_string());
        }

        rules
    }

    /// Parses priority from environment variable format (rule=priority pairs).
    fn parse_priority_env(val: &str) -> HashMap<String, i32> {
        let mut priority = HashMap::new();

        for part in val.split(':') {
            if let Some((rule, prio_str)) = part.split_once('=') {
                if let Ok(prio) = prio_str.parse() {
                    priority.insert(rule.to_string(), prio);
                }
            }
        }

        priority
    }

    /// Checks if a rule is enabled.
    pub fn is_rule_enabled(&self, rule_name: &str, enabled_by_default: bool) -> bool {
        // Check if explicitly excluded
        if self.exclude_rules.contains(&rule_name.to_string()) {
            return false;
        }

        // Check if explicitly enabled
        if self.rules.contains(&rule_name.to_string()) {
            return true;
        }

        // Check if all rules are enabled and this rule is enabled by default
        if self.rules.contains(&ALL_ENABLED.to_string()) && enabled_by_default {
            return true;
        }

        false
    }

    /// Gets the effective priority for a rule.
    pub fn get_rule_priority(&self, rule_name: &str, default: i32) -> i32 {
        self.priority
            .get(rule_name)
            .copied()
            .unwrap_or(default)
    }

    /// Checks if a command is a slow command.
    pub fn is_slow_command(&self, command: &str) -> bool {
        let cmd_parts: Vec<&str> = command.split_whitespace().collect();
        if cmd_parts.is_empty() {
            return false;
        }

        let cmd_name = cmd_parts[0];
        self.slow_commands.iter().any(|slow| {
            slow == cmd_name || slow == command
        })
    }

    /// Returns the timeout for a command.
    pub fn get_timeout(&self, command: &str) -> u64 {
        if self.is_slow_command(command) {
            self.wait_slow_command
        } else {
            self.wait_command
        }
    }
}

/// Partial settings for loading from TOML (all fields optional).
#[derive(Debug, Deserialize)]
struct SettingsPartial {
    rules: Option<Vec<String>>,
    exclude_rules: Option<Vec<String>>,
    wait_command: Option<u64>,
    wait_slow_command: Option<u64>,
    require_confirmation: Option<bool>,
    no_colors: Option<bool>,
    debug: Option<bool>,
    priority: Option<HashMap<String, i32>>,
    history_limit: Option<Option<usize>>,
    alter_history: Option<bool>,
    slow_commands: Option<Vec<String>>,
    repeat: Option<bool>,
    instant_mode: Option<bool>,
    num_close_matches: Option<usize>,
    env: Option<HashMap<String, String>>,
    excluded_search_path_prefixes: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.rules, vec!["ALL"]);
        assert!(settings.exclude_rules.is_empty());
        assert_eq!(settings.wait_command, 3);
        assert_eq!(settings.wait_slow_command, 15);
        assert!(settings.require_confirmation);
        assert!(!settings.no_colors);
        assert!(!settings.debug);
        assert_eq!(settings.num_close_matches, 3);
    }

    #[test]
    fn test_parse_rules_env() {
        let rules = Settings::parse_rules_env("sudo:git_push:DEFAULT_RULES");
        assert!(rules.contains(&"sudo".to_string()));
        assert!(rules.contains(&"git_push".to_string()));
        assert!(rules.contains(&"ALL".to_string()));
    }

    #[test]
    fn test_parse_priority_env() {
        let priority = Settings::parse_priority_env("sudo=500:git_push=100");
        assert_eq!(priority.get("sudo"), Some(&500));
        assert_eq!(priority.get("git_push"), Some(&100));
    }

    #[test]
    fn test_is_rule_enabled() {
        let mut settings = Settings::default();

        // All rules enabled by default
        assert!(settings.is_rule_enabled("sudo", true));
        assert!(!settings.is_rule_enabled("disabled_rule", false));

        // Exclude a rule
        settings.exclude_rules.push("sudo".to_string());
        assert!(!settings.is_rule_enabled("sudo", true));

        // Explicitly enable a rule
        settings.rules.push("special_rule".to_string());
        assert!(settings.is_rule_enabled("special_rule", false));
    }

    #[test]
    fn test_is_slow_command() {
        let settings = Settings::default();
        assert!(settings.is_slow_command("gradle build"));
        assert!(settings.is_slow_command("./gradlew test"));
        assert!(!settings.is_slow_command("git push"));
    }

    #[test]
    fn test_get_timeout() {
        let settings = Settings::default();
        assert_eq!(settings.get_timeout("git push"), 3);
        assert_eq!(settings.get_timeout("gradle build"), 15);
    }

    #[test]
    fn test_merge_from_args() {
        let mut settings = Settings::default();
        assert!(!settings.debug);
        assert!(settings.require_confirmation);

        settings.merge_from_args(true, false, true);

        assert!(settings.debug);
        assert!(!settings.require_confirmation);
    }

    #[test]
    fn test_toml_parsing() {
        let toml_content = r#"
            rules = ["sudo", "git_push"]
            wait_command = 5
            debug = true
        "#;

        let partial: SettingsPartial = toml::from_str(toml_content).unwrap();
        assert_eq!(partial.rules, Some(vec!["sudo".to_string(), "git_push".to_string()]));
        assert_eq!(partial.wait_command, Some(5));
        assert_eq!(partial.debug, Some(true));
    }
}
