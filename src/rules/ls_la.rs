use crate::types::{Command, Rule};

/// Rule that fixes common ls flag typos like `ls -la.` or `ls l`.
pub struct LsLaRule;

impl LsLaRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LsLaRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for LsLaRule {
    fn name(&self) -> &str {
        "ls_la"
    }

    fn matches(&self, command: &Command) -> bool {
        let script = command.script.trim();

        // Match various typos
        script == "ls l"
            || script == "ls -la."
            || script == "ls -al."
            || script == "ls la"
            || script == "sl"
            || script == "sl -la"
    }

    fn get_new_command(&self, command: &Command) -> Vec<String> {
        let script = command.script.trim();

        let new_command = match script {
            "ls l" | "ls la" => "ls -la",
            "ls -la." | "ls -al." => "ls -la",
            "sl" => "ls",
            "sl -la" => "ls -la",
            _ => return vec![],
        };

        vec![new_command.to_string()]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls_la_matches_ls_l() {
        let rule = LsLaRule::new();
        let cmd = Command::new("ls l", None);
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_ls_la_matches_sl() {
        let rule = LsLaRule::new();
        let cmd = Command::new("sl", None);
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_ls_la_matches_ls_la_dot() {
        let rule = LsLaRule::new();
        let cmd = Command::new("ls -la.", None);
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_ls_la_no_match_correct() {
        let rule = LsLaRule::new();
        let cmd = Command::new("ls -la", None);
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_ls_la_get_new_command_ls_l() {
        let rule = LsLaRule::new();
        let cmd = Command::new("ls l", None);
        assert_eq!(rule.get_new_command(&cmd), vec!["ls -la"]);
    }

    #[test]
    fn test_ls_la_get_new_command_sl() {
        let rule = LsLaRule::new();
        let cmd = Command::new("sl", None);
        assert_eq!(rule.get_new_command(&cmd), vec!["ls"]);
    }

    #[test]
    fn test_ls_la_no_output_required() {
        let rule = LsLaRule::new();
        assert!(!rule.requires_output());
    }
}
