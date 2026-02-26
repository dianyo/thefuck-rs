use crate::types::{Command, Rule};
use regex::Regex;

/// Rule that creates a directory and cd's into it when cd fails because
/// the directory doesn't exist.
pub struct CdMkdirRule;

impl CdMkdirRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CdMkdirRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CdMkdirRule {
    fn name(&self) -> &str {
        "cd_mkdir"
    }

    fn matches(&self, command: &Command) -> bool {
        if !command.script.starts_with("cd ") {
            return false;
        }

        let output = match &command.output {
            Some(out) => out.to_lowercase(),
            None => return false,
        };

        output.contains("no such file or directory")
            || output.contains("cd: can't cd to")
            || output.contains("does not exist")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Extract the directory from `cd <dir>`
        let re = Regex::new(r"^cd\s+(.*)$").unwrap();

        if let Some(caps) = re.captures(&command.script) {
            let dir = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            // Return: mkdir -p <dir> && cd <dir>
            vec![format!("mkdir -p {} && cd {}", dir, dir)]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_mkdir_matches_no_such_file() {
        let rule = CdMkdirRule::new();
        let cmd = Command::new(
            "cd myproject",
            Some("cd: no such file or directory: myproject".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cd_mkdir_matches_does_not_exist() {
        let rule = CdMkdirRule::new();
        let cmd = Command::new(
            "cd newdir",
            Some("cd: The directory 'newdir' does not exist".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cd_mkdir_no_match_not_cd() {
        let rule = CdMkdirRule::new();
        let cmd = Command::new(
            "mkdir newdir",
            Some("No such file or directory".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_cd_mkdir_no_match_success() {
        let rule = CdMkdirRule::new();
        let cmd = Command::new("cd existingdir", Some("".to_string()));
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_cd_mkdir_get_new_command() {
        let rule = CdMkdirRule::new();
        let cmd = Command::new(
            "cd myproject",
            Some("no such file or directory".to_string()),
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["mkdir -p myproject && cd myproject"]
        );
    }

    #[test]
    fn test_cd_mkdir_get_new_command_with_path() {
        let rule = CdMkdirRule::new();
        let cmd = Command::new(
            "cd /path/to/newdir",
            Some("no such file or directory".to_string()),
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(
            new_commands,
            vec!["mkdir -p /path/to/newdir && cd /path/to/newdir"]
        );
    }
}
