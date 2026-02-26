use super::{ShellOperations, ShellType};
use crate::config::Settings;
use crate::error::Result;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Zsh shell implementation.
pub struct Zsh {
    settings: Settings,
}

impl Zsh {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Parses an alias line from zsh alias output.
    /// Format: name='value' or name="value"
    fn parse_alias(alias: &str) -> Option<(String, String)> {
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
        raw_aliases.lines().filter_map(Self::parse_alias).collect()
    }

    /// Gets the history file path.
    fn get_history_file(&self) -> PathBuf {
        env::var("HISTFILE").map(PathBuf::from).unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".zsh_history"))
                .unwrap_or_else(|| PathBuf::from("~/.zsh_history"))
        })
    }

    /// Extracts command from zsh history line.
    /// Zsh history format: `: timestamp:0;command`
    fn script_from_history(line: &str) -> Option<String> {
        if line.contains(';') {
            Some(line.split_once(';')?.1.to_string())
        } else {
            None
        }
    }
}

impl ShellOperations for Zsh {
    fn shell_type(&self) -> ShellType {
        ShellType::Zsh
    }

    fn app_alias(&self, alias_name: &str) -> String {
        let alter_history = if self.settings.alter_history {
            "test -n \"$TF_CMD\" && print -s $TF_CMD"
        } else {
            ""
        };

        format!(
            r#"{name} () {{
    TF_PYTHONIOENCODING=$PYTHONIOENCODING;
    export TF_SHELL=zsh;
    export TF_ALIAS={name};
    TF_SHELL_ALIASES=$(alias);
    export TF_SHELL_ALIASES;
    TF_HISTORY="$(fc -ln -10)";
    export TF_HISTORY;
    export PYTHONIOENCODING=utf-8;
    TF_CMD=$(
        thefuck --force-command "$TF_HISTORY" $@
    ) && eval $TF_CMD;
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
            .map_while(|l| l.ok())
            .filter_map(|l| Self::script_from_history(&l))
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
        // In zsh, history is updated via the shell alias using `print -s`
        tracing::debug!("Would add to zsh history: {}", command);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_from_history() {
        let line = ": 1609459200:0;git push origin main";
        let result = Zsh::script_from_history(line);
        assert_eq!(result, Some("git push origin main".to_string()));
    }

    #[test]
    fn test_script_from_history_no_semicolon() {
        let line = "simple command";
        let result = Zsh::script_from_history(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_alias() {
        let result = Zsh::parse_alias("ll='ls -la'");
        assert_eq!(result, Some(("ll".to_string(), "ls -la".to_string())));
    }
}
