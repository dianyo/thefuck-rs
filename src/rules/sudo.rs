use crate::types::{Command, Rule};

/// Patterns that indicate permission denied errors.
const PERMISSION_PATTERNS: &[&str] = &[
    "permission denied",
    "eacces",
    "pkg: insufficient privileges",
    "you cannot perform this operation unless you are root",
    "non-root users cannot",
    "operation not permitted",
    "not super-user",
    "superuser privilege",
    "root privilege",
    "this command has to be run under the root user.",
    "this operation requires root.",
    "requested operation requires superuser privilege",
    "must be run as root",
    "must run as root",
    "must be superuser",
    "must be root",
    "need to be root",
    "need root",
    "needs to be run as root",
    "only root can ",
    "you don't have access to the history db.",
    "authentication is required",
    "edspermissionerror",
    "you don't have write permissions",
    "use `sudo`",
    "sudorequirederror",
    "error: insufficient privileges",
    "updatedb: can not open a temporary file",
];

/// Rule that prepends `sudo` to commands that fail due to permission errors.
pub struct SudoRule;

impl SudoRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SudoRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for SudoRule {
    fn name(&self) -> &str {
        "sudo"
    }

    fn matches(&self, command: &Command) -> bool {
        let output = match &command.output {
            Some(out) => out.to_lowercase(),
            None => return false,
        };

        // Don't match if already using sudo (unless && is present)
        let mut cmd = command.clone();
        let script_parts = cmd.script_parts();
        if !script_parts.is_empty() && script_parts[0] == "sudo" && !command.script.contains("&&") {
            return false;
        }

        // Check if any permission pattern matches
        PERMISSION_PATTERNS
            .iter()
            .any(|pattern| output.contains(pattern))
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let script = &command.script;

        if script.contains("&&") {
            // For commands with &&, wrap in sh -c
            let clean_script = script.replace("sudo ", "");
            vec![format!("sudo sh -c \"{}\"", clean_script)]
        } else if script.contains('>') {
            // For redirections, wrap in sh -c
            let escaped = script.replace('"', "\\\"");
            vec![format!("sudo sh -c \"{}\"", escaped)]
        } else {
            // Simple case: just prepend sudo
            vec![format!("sudo {}", script)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sudo_matches_permission_denied() {
        let rule = SudoRule::new();
        let cmd = Command::new(
            "cat /etc/shadow",
            Some("cat: /etc/shadow: Permission denied".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_sudo_matches_operation_not_permitted() {
        let rule = SudoRule::new();
        let cmd = Command::new(
            "rm /protected/file",
            Some("rm: cannot remove '/protected/file': Operation not permitted".to_string()),
        );
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_sudo_no_match_already_sudo() {
        let rule = SudoRule::new();
        let cmd = Command::new(
            "sudo cat /etc/shadow",
            Some("sudo: command not found".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_sudo_no_match_no_permission_error() {
        let rule = SudoRule::new();
        let cmd = Command::new(
            "git push",
            Some("error: failed to push some refs".to_string()),
        );
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_sudo_get_new_command_simple() {
        let rule = SudoRule::new();
        let cmd = Command::new("cat /etc/shadow", Some("Permission denied".to_string()));
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["sudo cat /etc/shadow"]);
    }

    #[test]
    fn test_sudo_get_new_command_with_redirect() {
        let rule = SudoRule::new();
        let cmd = Command::new(
            "echo 'test' > /etc/file",
            Some("Permission denied".to_string()),
        );
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["sudo sh -c \"echo 'test' > /etc/file\""]);
    }

    #[test]
    fn test_sudo_get_new_command_with_and() {
        let rule = SudoRule::new();
        let cmd = Command::new("cd /root && ls", Some("Permission denied".to_string()));
        let new_commands = rule.get_new_command(&cmd);
        assert_eq!(new_commands, vec!["sudo sh -c \"cd /root && ls\""]);
    }
}
