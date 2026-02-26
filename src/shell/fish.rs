use super::{ShellOperations, ShellType};
use crate::config::Settings;
use crate::error::Result;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Fish shell implementation.
pub struct Fish {
    settings: Settings,
}

impl Fish {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Gets the history file path.
    fn get_history_file(&self) -> PathBuf {
        dirs::config_dir()
            .map(|c| c.join("fish/fish_history"))
            .unwrap_or_else(|| PathBuf::from("~/.config/fish/fish_history"))
    }

    /// Extracts command from fish history line.
    /// Fish history format: `- cmd: command`
    fn script_from_history(line: &str) -> Option<String> {
        if line.contains("- cmd: ") {
            Some(line.split("- cmd: ").nth(1)?.to_string())
        } else {
            None
        }
    }

    /// Formats a history entry for fish.
    fn format_history_line(command: &str) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        format!("- cmd: {}\n  when: {}\n", command, timestamp)
    }
}

impl ShellOperations for Fish {
    fn shell_type(&self) -> ShellType {
        ShellType::Fish
    }

    fn app_alias(&self, alias_name: &str) -> String {
        let alter_history = if self.settings.alter_history {
            "    builtin history delete --exact --case-sensitive -- $fucked_up_command\n    builtin history merge\n"
        } else {
            ""
        };

        format!(
            r#"function {name} -d "Correct your previous console command"
    set -l fucked_up_command $history[1]
    set -lx TF_SHELL fish
    set -lx TF_ALIAS {name}
    set -lx PYTHONIOENCODING utf-8
    thefuck --force-command "$fucked_up_command" $argv | read -l unfucked_command
    if test -n "$unfucked_command"
        eval $unfucked_command
{alter_history}    end
end"#,
            name = alias_name,
            alter_history = alter_history
        )
    }

    fn quote(&self, command: &str) -> String {
        // Fish uses different escaping
        let escaped = command
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$");
        format!("\"{}\"", escaped)
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
            .filter_map(|l| Self::script_from_history(&l))
            .filter(|l| !l.is_empty())
            .collect();

        // Return last `limit` lines
        let start = lines.len().saturating_sub(limit);
        Ok(lines[start..].to_vec())
    }

    fn expand_aliases(&self, command: &str) -> String {
        // Fish aliases are more complex - they can be functions
        // For now, we just return the command as-is
        // TODO: Implement fish function/alias expansion
        command.to_string()
    }

    fn put_to_history(&self, command: &str) -> Result<()> {
        let history_file = self.get_history_file();

        if history_file.exists() {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&history_file)?;
            let entry = Self::format_history_line(command);
            file.write_all(entry.as_bytes())?;
        }

        Ok(())
    }

    fn or_commands(&self, first: &str, second: &str) -> String {
        format!("{}; or {}", first, second)
    }

    fn and_commands(&self, first: &str, second: &str) -> String {
        format!("{}; and {}", first, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_from_history() {
        let line = "- cmd: git push origin main";
        let result = Fish::script_from_history(line);
        assert_eq!(result, Some("git push origin main".to_string()));
    }

    #[test]
    fn test_script_from_history_when_line() {
        let line = "  when: 1609459200";
        let result = Fish::script_from_history(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_or_commands() {
        let fish = Fish::new(Settings::default());
        let result = fish.or_commands("cmd1", "cmd2");
        assert_eq!(result, "cmd1; or cmd2");
    }

    #[test]
    fn test_and_commands() {
        let fish = Fish::new(Settings::default());
        let result = fish.and_commands("cmd1", "cmd2");
        assert_eq!(result, "cmd1; and cmd2");
    }
}
