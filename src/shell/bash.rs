use super::{ShellOperations, ShellType};
use crate::config::Settings;
use crate::error::Result;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Bash shell implementation.
pub struct Bash {
    settings: Settings,
}

impl Bash {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Parses an alias line from bash alias output.
    /// Format: alias name='value' or alias name="value"
    fn parse_alias(alias: &str) -> Option<(String, String)> {
        let alias = alias.strip_prefix("alias ").unwrap_or(alias);
        let (name, value) = alias.split_once('=')?;

        let value = value.trim();
        let value = if (value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\''))
        {
            &value[1..value.len() - 1]
        } else {
            value
        };

        Some((name.to_string(), value.to_string()))
    }

    /// Gets aliases from TF_SHELL_ALIASES environment variable.
    fn get_aliases_from_env() -> HashMap<String, String> {
        let raw_aliases = env::var("TF_SHELL_ALIASES").unwrap_or_default();
        raw_aliases
            .lines()
            .filter_map(Self::parse_alias)
            .collect()
    }

    /// Gets the history file path.
    fn get_history_file(&self) -> PathBuf {
        env::var("HISTFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|h| h.join(".bash_history"))
                    .unwrap_or_else(|| PathBuf::from("~/.bash_history"))
            })
    }
}

impl ShellOperations for Bash {
    fn shell_type(&self) -> ShellType {
        ShellType::Bash
    }

    fn app_alias(&self, alias_name: &str) -> String {
        let alter_history = if self.settings.alter_history {
            "history -s $TF_CMD;"
        } else {
            ""
        };

        format!(
            r#"function {name} () {{
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL=bash;
    export TF_ALIAS={name};
    export TF_SHELL_ALIASES=$(alias);
    export TF_HISTORY="$(fc -ln -10)";
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        thefuck --force-command "$TF_HISTORY" "$@"
    ) && eval "$TF_CMD";
    unset TF_HISTORY;
    export PYTHONIOENCODING=$TF_PYTHONIOENCODING;
    {alter_history}
}}"#,
            name = alias_name,
            alter_history = alter_history
        )
    }

    fn quote(&self, command: &str) -> String {
        shell_escape::escape(command.into()).to_string()
    }

    fn get_history(&self, limit: usize) -> Result<Vec<String>> {
        let history_file = self.get_history_file();

        if !history_file.exists() {
            return Ok(vec![]);
        }

        let file = fs::File::open(&history_file)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| !l.is_empty())
            .collect();

        // Return last `limit` lines
        let start = lines.len().saturating_sub(limit);
        Ok(lines[start..].to_vec())
    }

    fn expand_aliases(&self, command: &str) -> String {
        let aliases = Self::get_aliases_from_env();
        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        let binary = parts[0];

        if let Some(expanded) = aliases.get(binary) {
            if parts.len() > 1 {
                format!("{} {}", expanded, parts[1])
            } else {
                expanded.clone()
            }
        } else {
            command.to_string()
        }
    }

    fn put_to_history(&self, command: &str) -> Result<()> {
        // In bash, history is updated via the shell alias using `history -s`
        // We don't need to write to the file directly
        tracing::debug!("Would add to bash history: {}", command);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_alias_single_quotes() {
        let result = Bash::parse_alias("alias ll='ls -la'");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }

    #[test]
    fn test_parse_alias_double_quotes() {
        let result = Bash::parse_alias("alias ll=\"ls -la\"");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }

    #[test]
    fn test_parse_alias_no_quotes() {
        let result = Bash::parse_alias("alias ll=ls");
        assert_eq!(result, Some(("ll".to_string(), "ls".to_string())));
    }
}
