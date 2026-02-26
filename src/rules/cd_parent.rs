use crate::types::{Command, Rule};

/// Rule that fixes `cd..` to `cd ..` (missing space).
pub struct CdParentRule;

impl CdParentRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CdParentRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for CdParentRule {
    fn name(&self) -> &str {
        "cd_parent"
    }

    fn matches(&self, command: &Command) -> bool {
        command.script.trim() == "cd.."
    }

    fn get_new_command(&self, _command: &Command) -> Vec<String> {
        vec!["cd ..".to_string()]
    }

    fn requires_output(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cd_parent_matches() {
        let rule = CdParentRule::new();
        let cmd = Command::new("cd..", None);
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cd_parent_matches_with_whitespace() {
        let rule = CdParentRule::new();
        let cmd = Command::new("cd..  ", None);
        assert!(rule.matches(&cmd));
    }

    #[test]
    fn test_cd_parent_no_match_correct() {
        let rule = CdParentRule::new();
        let cmd = Command::new("cd ..", None);
        assert!(!rule.matches(&cmd));
    }

    #[test]
    fn test_cd_parent_get_new_command() {
        let rule = CdParentRule::new();
        let cmd = Command::new("cd..", None);
        assert_eq!(rule.get_new_command(&cmd), vec!["cd .."]);
    }

    #[test]
    fn test_cd_parent_no_output_required() {
        let rule = CdParentRule::new();
        assert!(!rule.requires_output());
    }
}
