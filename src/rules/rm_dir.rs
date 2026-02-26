use crate::types::{Command, Rule};

/// Rule that adds -r flag to rm when trying to remove a directory.
pub struct RmDirRule;

impl RmDirRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RmDirRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for RmDirRule {
    fn name(&self) -> &str {
        "rm_dir"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(out) => out.to_lowercase(),
            None => return false,
        };

        command.script.starts_with("rm ")
            && (output.contains("is a directory") || output.contains("cannot remove"))
            && !command.script.contains("-r")
            && !command.script.contains("-R")
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        // Insert -r after rm
        let script = command.script.replacen("rm ", "rm -r ", 1);
        vec![script]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rm_dir_matches() {
        let rule = RmDirRule::new();
        let cmd = Command::new(
            "rm mydir",
            Some("rm: mydir: is a directory".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_rm_dir_no_match_with_r_flag() {
        let rule = RmDirRule::new();
        let cmd = Command::new(
            "rm -r mydir",
            Some("rm: mydir: is a directory".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_rm_dir_no_match_file() {
        let rule = RmDirRule::new();
        let cmd = Command::new(
            "rm myfile",
            Some("rm: myfile: No such file or directory".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_rm_dir_get_new_command() {
        let rule = RmDirRule::new();
        let cmd = Command::new(
            "rm mydir",
            Some("is a directory".to_string()),
        );
        assert_eq!(rule.get_new_command(&cmd), vec!["rm -r mydir"]);
    }

    #[test]
    fn test_rm_dir_get_new_command_with_force() {
        let rule = RmDirRule::new();
        let cmd = Command::new(
            "rm -f mydir",
            Some("is a directory".to_string()),
        );
        assert_eq!(rule.get_new_command(&cmd), vec!["rm -r -f mydir"]);
    }
}
